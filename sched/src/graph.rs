extern crate spinlock;
extern crate xnor_llist;

use base::{SchedCall, TimeResched};
use binding::BindingGetP;
use context::{ChildContext, SchedContext};
use std;
use std::sync::Arc;
use xnor_llist::List;
use xnor_llist::Node as LNode;

pub trait GraphExec: Send {
    fn exec(&mut self, context: &mut dyn SchedContext) -> bool;
    fn child_append(&mut self, child: AChildP);
}

pub type ANodeP = Arc<spinlock::Mutex<dyn GraphExec>>;
pub type AChildP = Box<LNode<ANodeP>>;
pub type ChildList = List<ANodeP>;

pub type Micro = f32;
pub struct RootClock {
    children: ChildList,
    tick: usize,
    tick_sub: f32,
    period_micros: BindingGetP<Micro>,
}

pub struct FuncWrapper<F> {
    children: ChildList,
    func: Box<F>,
}

impl RootClock {
    pub fn new(period_micros: BindingGetP<Micro>) -> Self {
        Self {
            children: List::new(),
            tick: 0,
            tick_sub: 0f32,
            period_micros,
        }
    }

    pub fn child_append(&mut self, child: AChildP) {
        self.children.push_back(child);
    }
}

impl SchedCall for RootClock {
    fn sched_call(&mut self, context: &mut dyn SchedContext) -> TimeResched {
        let period_micros = self.period_micros.get();
        if self.children.count() > 0 {
            let mut ccontext = ChildContext::new(context, self.tick, period_micros);
            let mut tmp = List::new();
            std::mem::swap(&mut self.children, &mut tmp);

            for c in tmp.into_iter() {
                if c.lock().exec(&mut ccontext) {
                    self.children.push_back(c);
                }
            }
        }

        let ctp = context.context_tick_period_micros();
        if period_micros <= 0f32 || ctp <= 0f32 {
            TimeResched::ContextRelative(1)
        } else {
            let next = self.tick_sub + (period_micros / ctp);
            self.tick_sub = next.fract();
            self.tick += 1;

            //XXX what if next is less than 1?
            assert!(next >= 1f32, "tick less than sample size not supported");
            TimeResched::ContextRelative(std::cmp::max(1, next.floor() as usize))
        }
    }
}

impl<F> FuncWrapper<F>
where
    F: Fn(&mut dyn SchedContext, &mut ChildList) -> bool + Send,
{
    pub fn new_p(func: F) -> Arc<spinlock::Mutex<Self>> {
        Arc::new(spinlock::Mutex::new(Self {
            func: Box::new(func),
            children: List::new(),
        }))
    }
}

impl<F> GraphExec for FuncWrapper<F>
where
    F: Fn(&mut dyn SchedContext, &mut ChildList) -> bool + Send,
{
    fn exec(&mut self, context: &mut dyn SchedContext) -> bool {
        (self.func)(context, &mut self.children)
    }

    fn child_append(&mut self, child: AChildP) {
        self.children.push_back(child);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use base::{LList, Sched, Scheduler, SrcSink, TimeSched};
    use binding::{ParamBindingSet, SpinlockParamBinding};
    use context::{RootContext, SchedContext};
    use std;
    use std::thread;

    struct X {
        children: ChildList,
    }
    struct Y {}

    impl GraphExec for X {
        fn exec(&mut self, context: &mut dyn SchedContext) -> bool {
            println!("XES");

            let mut tmp = List::new();
            std::mem::swap(&mut self.children, &mut tmp);
            for c in tmp.into_iter() {
                if c.lock().exec(context) {
                    self.children.push_back(c);
                }
            }

            self.children.count() > 0
        }
        fn child_append(&mut self, child: AChildP) {
            self.children.push_back(child);
        }
    }

    impl GraphExec for Y {
        fn exec(&mut self, _context: &mut dyn SchedContext) -> bool {
            println!("ONCE");
            false
        }
        fn child_append(&mut self, _child: AChildP) {}
    }

    #[test]
    fn works() {
        type M<T> = spinlock::Mutex<T>;

        let x = Arc::new(M::new(X {
            children: List::new(),
        }));
        let y = Arc::new(M::new(Y {}));

        let mut l: LList<std::sync::Arc<M<dyn GraphExec>>> = List::new();
        l.push_back(LNode::new_boxed(x.clone()));
        x.lock().child_append(LNode::new_boxed(y.clone()));

        let mut src_sink = SrcSink::new();
        let mut list = LList::new();
        let mut trig_list = List::new();

        let mut c = RootContext::new(0, 0, &mut list, &mut trig_list, &mut src_sink);

        for i in l.iter() {
            i.lock().exec(&mut c);
        }

        for i in l.iter() {
            i.lock().exec(&mut c);
        }
    }

    struct TickStore {
        tick: Option<usize>,
    }

    impl TickStore {
        fn tick(&self) -> Option<usize> {
            self.tick
        }
    }

    impl Default for TickStore {
        fn default() -> Self {
            TickStore { tick: None }
        }
    }

    impl GraphExec for TickStore {
        fn exec(&mut self, context: &mut dyn SchedContext) -> bool {
            self.tick = Some(context.context_tick());
            true
        }
        fn child_append(&mut self, _child: AChildP) {}
    }

    #[test]
    fn scheduled() {
        let mut s = Scheduler::new();
        s.spawn_helper_threads();

        let e = s.executor();

        let mut clock_period = Arc::new(SpinlockParamBinding::new(1_000_000f32));
        let mut clock = Box::new(RootClock::new(clock_period.clone()));
        let tick_store = Arc::new(spinlock::Mutex::new(TickStore::default()));

        assert!(tick_store.lock().tick().is_none());
        clock.child_append(LNode::new_boxed(tick_store.clone()));
        assert!(tick_store.lock().tick().is_none());

        s.schedule(TimeSched::Relative(0), clock);

        let child = thread::spawn(move || {
            let mut e = e.unwrap();
            e.run(44100, 44100); //just on the verge of next tick
            assert_eq!(Some(0), tick_store.lock().tick());

            e.run(1, 44100); //next tick
            assert_eq!(Some(1), tick_store.lock().tick());

            e.run(44098, 44100);
            assert_eq!(Some(1), tick_store.lock().tick());

            e.run(1, 44100);
            assert_eq!(Some(1), tick_store.lock().tick());

            clock_period.set(500_000f32); //2x as fast

            e.run(2, 44100); //still waiting for next tick
            assert_eq!(Some(2), tick_store.lock().tick());

            e.run(44100, 44100);
            assert_eq!(Some(4), tick_store.lock().tick());
        });
        assert!(child.join().is_ok());
    }

}
