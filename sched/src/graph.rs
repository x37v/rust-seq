extern crate spinlock;
extern crate xnor_llist;

use base::{ChildContext, ParamBinding, SchedCall, SchedContext, TimeResched};
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
    period_micros: Arc<dyn ParamBinding<Micro>>,
}

impl RootClock {
    pub fn new(period_micros: Arc<dyn ParamBinding<Micro>>) -> Self {
        Self {
            children: List::new(),
            tick: 0,
            tick_sub: 0f32,
            period_micros,
        }
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

        if period_micros <= 0f32 {
            TimeResched::ContextRelative(1)
        } else {
            let next = self.tick_sub + (context.context_tick_period_micros() * period_micros);
            self.tick_sub = next.fract();
            self.tick += 1;

            //XXX what if next is less than 1?
            assert!(next >= 1f32, "tick less than sample size not supported");

            TimeResched::ContextRelative(std::cmp::max(1, next.floor() as usize))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use base::{LList, RootContext, Scheduler, SpinlockParamBinding, SrcSink, TimeSched};
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

        let mut c = RootContext::new(0, 0, &mut list, &mut src_sink);

        for i in l.iter() {
            i.lock().exec(&mut c);
        }

        for i in l.iter() {
            i.lock().exec(&mut c);
        }
    }

    #[test]
    fn scheduled() {
        let mut s = Scheduler::new();
        s.spawn_helper_threads();

        let e = s.executor();

        let mut clock_period = Arc::new(SpinlockParamBinding::new(1_000_000f32));
        let clock = Box::new(RootClock::new(clock_period));

        s.schedule(TimeSched::Relative(0), clock);

        let child = thread::spawn(move || {
            let mut e = e.unwrap();
            e.run(44100, 44100);
            e.run(44100, 44100);
        });
    }

}
