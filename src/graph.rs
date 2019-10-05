pub mod clock_ratio;
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
