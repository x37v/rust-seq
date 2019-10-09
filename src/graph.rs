use crate::event::EventEvalContext;

pub mod clock_ratio;
pub mod nchild;
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
    } else {
pub struct GraphNodeContainer(&'static spin::Mutex<dyn GraphNode>);
    }
}

impl GraphNode for GraphNodeContainer {
    fn node_exec(&mut self, context: &mut dyn EventEvalContext) {
        self.0.lock().node_exec(context)
    }
}
