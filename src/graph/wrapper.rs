use crate::event::EventEvalContext;
use crate::graph::{GraphChildExec, GraphNode, GraphNodeExec};

/// A container to hold a `GraphNodeExec` and `GraphChildExec` that implements `GraphNode` to
/// call both.
pub struct GraphNodeWrapper<N, C, E>
where
    N: GraphNodeExec<E>,
    C: GraphChildExec<E>,
{
    pub node: N,
    pub children: C,
    _phantom: core::marker::PhantomData<E>,
}

impl<N, C, E> GraphNodeWrapper<N, C, E>
where
    N: GraphNodeExec<E>,
    C: GraphChildExec<E>,
{
    pub fn new(node: N, children: C) -> Self {
        Self {
            node,
            children,
            _phantom: Default::default(),
        }
    }
}

impl<N, C, E> GraphNode<E> for GraphNodeWrapper<N, C, E>
where
    N: GraphNodeExec<E>,
    C: GraphChildExec<E>,
{
    fn node_exec(&self, context: &mut dyn EventEvalContext<E>) {
        self.node.graph_exec(context, &self.children)
    }
}
