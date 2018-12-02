extern crate spinlock;
extern crate xnor_llist;

use base::{SchedCall, TimeResched};
use binding::BindingGetP;
use context::{ChildContext, SchedContext};
use ptr::{SShrPtr, UniqPtr};
use std;
use std::cmp::{Ordering, PartialOrd};
use xnor_llist::List as LList;
use xnor_llist::Node as LNode;

pub mod clock_ratio;
pub mod euclidean_gate;
pub mod func;
pub mod gate;
pub mod index_latch;
pub mod index_report;
pub mod midi;
pub mod node_wrapper;
pub mod one_hot;
pub mod root_clock;
pub mod step_seq;

pub trait GraphExec: Send {
    fn exec(&mut self, context: &mut dyn SchedContext, children: &mut dyn ChildExec) -> bool;
    fn children_max(&self) -> ChildCount;
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
    fn has_children(&self) -> bool;
}

pub trait GraphIndexExec: Send {
    fn exec_index(&mut self, index: usize, context: &mut dyn SchedContext);
}

pub trait GraphNode {
    fn exec(&mut self, context: &mut dyn SchedContext) -> bool;
    fn child_append(&mut self, child: AChildP) -> bool;
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum ChildCount {
    None,
    Some(usize),
    Inf,
}

struct Children<'a> {
    children: &'a mut ChildList,
}

struct NChildren<'a> {
    children: &'a mut ChildList,
    index_children: &'a mut IndexChildList,
}

pub type ANodeP = SShrPtr<dyn GraphNode>;
pub type AChildP = UniqPtr<LNode<ANodeP>>;
pub type ChildList = LList<ANodeP>;

pub type AIndexNodeP = SShrPtr<dyn GraphIndexExec>;
pub type AIndexChildP = UniqPtr<LNode<AIndexNodeP>>;
pub type IndexChildList = LList<AIndexNodeP>;

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

    fn has_children(&self) -> bool {
        self.children.count() > 0
    }
}

impl<'a> NChildren<'a> {
    pub fn new(children: &'a mut ChildList, index_children: &'a mut IndexChildList) -> Self {
        Self {
            children,
            index_children,
        }
    }

    fn exec_index_callbacks(&mut self, index: usize, context: &mut dyn SchedContext) {
        for c in self.index_children.iter() {
            c.lock().exec_index(index, context);
        }
    }
}

impl<'a> ChildExec for NChildren<'a> {
    fn exec(&mut self, context: &mut dyn SchedContext, index: usize) -> ChildCount {
        if let Some(c) = self.children.pop_front() {
            self.exec_index_callbacks(index, context);
            if c.lock().exec(context) {
                self.children.push_front(c);
            }
        }
        self.count()
    }

    fn exec_range(
        &mut self,
        context: &mut dyn SchedContext,
        range: std::ops::Range<usize>,
    ) -> ChildCount {
        if let Some(c) = self.children.pop_front() {
            let mut pop = true;
            {
                let mut l = c.lock();
                for index in range {
                    self.exec_index_callbacks(index, context);
                    pop |= !l.exec(context);
                }
            }
            if !pop {
                self.children.push_front(c);
            }
        }
        self.count()
    }

    fn exec_all(&mut self, context: &mut dyn SchedContext) -> ChildCount {
        self.exec(context, 0)
    }

    fn count(&self) -> ChildCount {
        if self.has_children() {
            ChildCount::Inf
        } else {
            ChildCount::None
        }
    }

    fn has_children(&self) -> bool {
        self.children.count() > 0
    }
}

#[cfg(test)]
mod tests {
    use super::node_wrapper::GraphNodeWrapper;
    use super::*;
    use base::{LList, SrcSink};
    use context::{RootContext, SchedContext};
    use std;

    struct X {}
    struct Y {}

    impl GraphExec for X {
        fn exec(&mut self, context: &mut dyn SchedContext, children: &mut dyn ChildExec) -> bool {
            println!("XES");
            children.exec_all(context);
            children.has_children()
        }

        fn children_max(&self) -> ChildCount {
            ChildCount::Inf
        }
    }

    impl GraphExec for Y {
        fn exec(&mut self, _context: &mut dyn SchedContext, _childen: &mut dyn ChildExec) -> bool {
            println!("ONCE");
            false
        }
        fn children_max(&self) -> ChildCount {
            ChildCount::Inf
        }
    }

    #[test]
    fn works() {
        let x = new_sshrptr!(GraphNodeWrapper::new(new_uniqptr!(X {})));
        let y = new_sshrptr!(GraphNodeWrapper::new(new_uniqptr!(Y {})));

        let mut l: LList<SShrPtr<dyn GraphNode>> = LList::new();
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

    /*
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
        fn exec(&mut self, context: &mut dyn SchedContext, _children: &mut dyn ChildExec) -> bool {
            self.tick = Some(context.context_tick());
            true
        }
        fn children_max(&self) -> ChildCount {
            ChildCount::Inf
        }
    }

    #[test]
    fn scheduled() {
        let mut s = Scheduler::new();
        s.spawn_helper_threads();

        let e = s.executor();

        let clock_period = new_shrptr!(SpinlockParamBinding::new(1_000_000f32));
        let mut clock = Box::new(RootClock::new(clock_period.clone()));
        let tick_store = GraphNodeWrapper::new_p(TickStore::default());

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
    */

}
