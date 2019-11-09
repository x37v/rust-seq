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

#[derive(Clone)]
pub struct IndexChildContainer(alloc::sync::Arc<spin::Mutex<dyn GraphIndexExec>>);

impl GraphNodeContainer {
    pub fn new<T: 'static + GraphNode>(item: T) -> Self {
        Self(alloc::sync::Arc::new(spin::Mutex::new(item)))
    }
}

impl GraphNode for GraphNodeContainer {
    fn node_exec(&mut self, context: &mut dyn EventEvalContext) {
        self.0.lock().node_exec(context)
    }
}

impl IndexChildContainer {
    pub fn new<T: 'static + GraphIndexExec>(item: T) -> Self {
        Self(alloc::sync::Arc::new(spin::Mutex::new(item)))
    }
}

impl GraphIndexExec for IndexChildContainer {
    fn exec_index(&mut self, index: usize, context: &mut dyn EventEvalContext) {
        self.0.lock().exec_index(index, context)
    }
}

#[cfg(test)]
mod tests {
    use super::node_wrapper::GraphNodeWrapper;
    use super::*;
    use std::thread;

    struct TestNodeExec;
    impl GraphNodeExec for TestNodeExec {
        fn graph_exec(
            &mut self,
            context: &mut dyn EventEvalContext,
            children: &mut dyn GraphChildExec,
        ) {
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
