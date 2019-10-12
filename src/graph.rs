use crate::event::EventEvalContext;

pub mod children;
pub mod clock_ratio;
pub mod node_wrapper;

mod traits;
pub use self::traits::*;

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum ChildCount {
    None,
    Some(usize),
    Inf,
}

cfg_if::cfg_if! {
    if #[cfg(feature = "graph_arc")] {
        extern crate alloc;
        pub struct GraphNodeContainer(alloc::sync::Arc<spin::Mutex<dyn GraphNode>>);
        pub struct IndexChildContainer(alloc::sync::Arc<spin::Mutex<dyn GraphIndexExec>>);
    } else {
        pub struct GraphNodeContainer(&'static spin::Mutex<dyn GraphNode>);
        pub struct IndexChildContainer(&'static spin::Mutex<dyn GraphIndexExec>);
    }
}

impl GraphNode for GraphNodeContainer {
    fn node_exec(&mut self, context: &mut dyn EventEvalContext) {
        self.0.lock().node_exec(context)
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
    use core::convert::From;
    use spin::Mutex;

    struct TestNodeExec;
    impl GraphNodeExec for TestNodeExec {
        fn graph_exec(
            &mut self,
            _context: &mut dyn EventEvalContext,
            _children: &mut dyn GraphChildExec,
        ) {
        }
    }

    cfg_if::cfg_if! {
        if #[cfg(not(feature = "graph_arc"))] {
            use crate::graph::children::empty::Children;
            static TEST_EXEC: Mutex<TestNodeExec> = Mutex::new(TestNodeExec);
            static TEST_CHILD: Mutex<TestNodeExec> = Mutex::new(TestNodeExec);
            static EMPTY: [Mutex<TestNodeExec>;0] = [];
            static NODE: GraphNodeWrapper<&'static Mutex<TestNodeExec>,Children> = GraphNodeWrapper{exec: &TEST_EXEC, children: Children };
        }
    }
}
