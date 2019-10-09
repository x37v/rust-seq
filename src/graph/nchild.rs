use crate::event::*;
use crate::graph::{ChildCount, GraphChildExec, GraphNode, GraphNodeContainer};
use core::marker::PhantomData;

/// A graph node that fakes that it has infinite children.
/// It has 'index' children that get called each time a child is addressed by index.

pub trait GraphIndexExec: Send {
    fn exec_index(&mut self, index: usize, context: &mut dyn EventEvalContext);
}

cfg_if::cfg_if! {
    if #[cfg(feature = "graph_arc")] {
        extern crate alloc;
        pub struct IndexChildContainer(alloc::sync::Arc<spin::Mutex<dyn GraphIndexExec>>);
    } else {
        pub struct IndexChildContainer(&'static spin::Mutex<dyn GraphIndexExec>);
    }
}

/*
impl GraphIndexExec for IndexChildContainer {
    fn exec_index(&mut self, index: usize, context: &mut dyn EventEvalContext) {
        self.0.lock().exec_index(index, context);
    }
}

pub struct IndexChildren([IndexChildContainer]);

pub struct NChildWrapper {
    child: GraphNodeContainer,
    index_children: IndexChildren,
}

impl GraphChildExec for NChildWrapper {
    fn child_count(&self) -> ChildCount {
        ChildCount::Inf
    }
    fn child_exec_range(
        &mut self,
        context: &mut dyn EventEvalContext,
        range: core::ops::Range<usize>,
    ) {
        self.child.exec(context);
        for i in range {
            for ic in &mut self.index_children.0 {
                ic.exec_index(i, context);
            }
        }
    }
}
*/
