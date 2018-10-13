extern crate spinlock;
extern crate xnor_llist;

use base::{SchedCall, TimeResched};
use binding::BindingGetP;
use context::{ChildContext, SchedContext};
use std;
use std::cmp::{Ordering, PartialOrd};
use std::sync::Arc;
use xnor_llist::List as LList;
use xnor_llist::Node as LNode;

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum ChildCount {
    None,
    Some(usize),
    Inf,
}

pub trait GraphExec: Send {
    fn exec(&mut self, context: &mut dyn SchedContext, children: &mut dyn ChildExec) -> bool;
    fn allowed_children(&self) -> ChildCount;
}

pub trait ChildExec {
    fn exec(&mut self, context: &mut dyn SchedContext, index: usize) -> ChildCount;
    fn exec_range(
        &mut self,
        context: &mut dyn SchedContext,
        range: std::ops::Range<usize>,
    ) -> ChildCount;
    fn exec_all(&mut self, context: &mut dyn SchedContext) -> ChildCount;
    fn count(&self) -> ChildCount;
}

pub trait GraphIndexContext: SchedContext {
    fn index(&self) -> usize; //the index that we were called with
    fn as_sched_context(&mut self) -> &mut SchedContext;
}

pub trait GraphIndexExec: Send {
    fn exec_index(&mut self, context: &mut dyn GraphIndexContext);
}

pub trait GraphNode {
    fn exec(&mut self, context: &mut dyn SchedContext) -> bool;
    fn child_append(&mut self, child: AChildP) -> bool;
}

pub struct GraphNodeWrapper {
    exec: Box<GraphExec>,
    children: ChildList,
}

pub struct Children<'a> {
    children: &'a mut ChildList,
}

pub type ANodeP = Arc<spinlock::Mutex<dyn GraphNode>>;
pub type AChildP = Box<LNode<ANodeP>>;
pub type ChildList = LList<ANodeP>;

pub type AIndexNodeP = Arc<spinlock::Mutex<dyn GraphIndexExec>>;
pub type AIndexChildP = Box<LNode<AIndexNodeP>>;
pub type IndexChildList = LList<AIndexNodeP>;

pub type Micro = f32;
pub struct RootClock {
    tick: usize,
    tick_sub: f32,
    period_micros: BindingGetP<Micro>,
    children: ChildList,
}

pub struct FuncWrapper<F> {
    func: Box<F>,
    max_children: ChildCount,
}

impl GraphNode for GraphNodeWrapper {
    fn exec(&mut self, context: &mut dyn SchedContext) -> bool {
        let mut children = Children::new(&mut self.children);
        self.exec.exec(context, &mut children)
    }
    fn child_append(&mut self, child: AChildP) -> bool {
        if match self.exec.allowed_children() {
            ChildCount::None => false,
            ChildCount::Some(v) => self.children.count() < v,
            ChildCount::Inf => true,
        } {
            self.children.push_back(child);
            true
        } else {
            false
        }
    }
}

impl GraphNodeWrapper {
    pub fn new_p(exec: Box<GraphExec>) -> Arc<spinlock::Mutex<Self>> {
        Arc::new(spinlock::Mutex::new(Self {
            exec,
            children: LList::new(),
        }))
    }
}

impl PartialOrd for ChildCount {
    fn partial_cmp(&self, other: &ChildCount) -> Option<Ordering> {
        match self {
            ChildCount::None => Some(match other {
                ChildCount::None | ChildCount::Some(0) => Ordering::Equal,
                _ => Ordering::Less,
            }),
            ChildCount::Inf => Some(match other {
                ChildCount::Inf => Ordering::Equal,
                _ => Ordering::Greater,
            }),
            ChildCount::Some(v) => match other {
                ChildCount::Inf => Some(Ordering::Less),
                ChildCount::None => Some(if v == &0usize {
                    Ordering::Equal
                } else {
                    Ordering::Greater
                }),
                ChildCount::Some(ov) => v.partial_cmp(&ov),
            },
        }
    }
}

impl<'a> Children<'a> {
    fn new(children: &'a mut ChildList) -> Self {
        Self { children }
    }
}

impl<'a> ChildExec for Children<'a> {
    fn exec(&mut self, context: &mut dyn SchedContext, index: usize) -> ChildCount {
        let tmp = self.children.split(|_| true); //XXX should be a better way
        for (i, c) in (0..).zip(tmp.into_iter()) {
            if i == index && !c.lock().exec(context) {
                continue;
            }
            self.children.push_back(c);
        }
        self.count()
    }

    fn exec_range(
        &mut self,
        context: &mut dyn SchedContext,
        range: std::ops::Range<usize>,
    ) -> ChildCount {
        let tmp = self.children.split(|_| true); //XXX should be a better way
        for (i, c) in (0..).zip(tmp.into_iter()) {
            if i.ge(&range.start) && i.lt(&range.end) && !c.lock().exec(context) {
                continue;
            }
            self.children.push_back(c);
        }
        self.count()
    }

    fn exec_all(&mut self, context: &mut dyn SchedContext) -> ChildCount {
        let tmp = self.children.split(|_| true); //XXX should be a better way
        for c in tmp.into_iter() {
            if c.lock().exec(context) {
                self.children.push_back(c);
            }
        }
        self.count()
    }
    fn count(&self) -> ChildCount {
        ChildCount::Some(self.children.count())
    }
}

impl RootClock {
    pub fn new(period_micros: BindingGetP<Micro>) -> Self {
        Self {
            tick: 0,
            tick_sub: 0f32,
            period_micros,
            children: LList::new(),
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
            let mut tmp = LList::new();
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
    F: Fn(&mut dyn SchedContext, &mut dyn ChildExec) -> bool + Send,
{
    pub fn new_boxed(max_children: ChildCount, func: F) -> Box<Self> {
        Box::new(Self {
            func: Box::new(func),
            max_children,
        })
    }
    /*
    pub fn new_p(max_children: ChildCount, func: F) -> Arc<spinlock::Mutex<GraphNodeWrapper>> {
        GraphNodeWrapper::new_p(Self::new_boxed(max_children, func))
    }
    */
}

impl<F> GraphExec for FuncWrapper<F>
where
    F: Fn(&mut dyn SchedContext, &mut dyn ChildExec) -> bool + Send,
{
    fn exec(&mut self, context: &mut dyn SchedContext, children: &mut dyn ChildExec) -> bool {
        (self.func)(context, children)
    }

    fn allowed_children(&self) -> ChildCount {
        self.max_children
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

            let mut tmp = LList::new();
            std::mem::swap(&mut self.children, &mut tmp);
            for c in tmp.into_iter() {
                if c.lock().exec(context) {
                    self.children.push_back(c);
                }
            }

            self.children.count() > 0
        }
    }

    impl GraphExec for Y {
        fn exec(&mut self, _context: &mut dyn SchedContext) -> bool {
            println!("ONCE");
            false
        }
    }

    #[test]
    fn works() {
        type M<T> = spinlock::Mutex<T>;

        let x = Arc::new(M::new(X {
            children: LList::new(),
        }));
        let y = Arc::new(M::new(Y {}));

        let mut l: LList<std::sync::Arc<M<dyn GraphExec>>> = LList::new();
        l.push_back(LNode::new_boxed(x.clone()));
        x.lock().child_append(LNode::new_boxed(y.clone()));

        let mut src_sink = SrcSink::new();
        let mut list = LList::new();
        let mut trig_list = LList::new();

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
    }

    #[test]
    fn scheduled() {
        let mut s = Scheduler::new();
        s.spawn_helper_threads();

        let e = s.executor();

        let clock_period = Arc::new(SpinlockParamBinding::new(1_000_000f32));
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
