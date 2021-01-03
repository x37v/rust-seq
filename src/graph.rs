use crate::event::EventEvalContext;
extern crate alloc;

pub mod bindstore;
pub mod children;
pub mod clock_ratio;
pub mod fanout;
pub mod func;
pub mod gate;
pub mod midi;
pub mod node_wrapper;
pub mod one_hot;
pub mod retrig_scheduler;
pub mod root_clock;
pub mod root_event;
pub mod root_wrapper;
pub mod step_seq;
pub mod tick_record;

mod traits;
pub use self::traits::*;

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum ChildCount {
    None,
    Some(usize),
    Inf,
}

#[derive(Clone)]
pub struct GraphNodeContainer(alloc::sync::Arc<spin::Mutex<dyn GraphNode>>);

impl GraphNodeContainer {
    pub fn new<T: 'static + GraphNode>(item: T) -> Self {
        Self(alloc::sync::Arc::new(spin::Mutex::new(item)))
    }
}

impl GraphNode for GraphNodeContainer {
    fn node_exec(&self, context: &mut dyn EventEvalContext) {
        self.0.lock().node_exec(context)
    }
}

/// No-op node impl for unit.
impl GraphNode for () {
    fn node_exec(&self, _context: &mut dyn EventEvalContext) {}
}

#[cfg(test)]
mod tests {
    use super::node_wrapper::GraphNodeWrapper;
    use super::*;
    use std::thread;

    struct TestNodeExec;
    impl GraphNodeExec for TestNodeExec {
        fn graph_exec(&self, context: &mut dyn EventEvalContext, children: &dyn GraphChildExec) {
            children.child_exec_all(context);
        }
    }

    #[test]
    fn can_build_and_exec() {
        let mut context = crate::context::tests::TestContext::new(0, 44100);
        let c = GraphNodeContainer::new(GraphNodeWrapper::new(
            TestNodeExec,
            crate::graph::children::empty::Children,
        ));

        let children = crate::graph::children::boxed::Children::new(Box::new([c]));

        let mut r = GraphNodeContainer::new(GraphNodeWrapper::new(TestNodeExec, children));

        r.node_exec(&mut context);
        r.node_exec(&mut context);
        let child = thread::spawn(move || {
            r.node_exec(&mut context);
        });
        assert!(child.join().is_ok());
    }
}
