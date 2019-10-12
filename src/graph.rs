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

extern crate alloc;
pub struct GraphNodeContainer(alloc::sync::Arc<spin::Mutex<dyn GraphNode>>);
pub struct IndexChildContainer(alloc::sync::Arc<spin::Mutex<dyn GraphIndexExec>>);

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
    type EmptyChildren = crate::graph::children::empty::Children;

    struct TestNodeExec;
    impl GraphNodeExec for TestNodeExec {
        fn graph_exec(
            &mut self,
            _context: &mut dyn EventEvalContext,
            _children: &mut dyn GraphChildExec,
        ) {
        }
    }
}
