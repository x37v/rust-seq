use crate::event::EventEvalContext;
use crate::graph::{GraphChildExec, GraphNode, GraphNodeExec};

/// A container to hold a `GraphNodeExec` and `GraphChildExec` that implements `GraphNode` to
/// call both.
pub struct GraphNodeWrapper<N, C, E, U>
where
    N: GraphNodeExec<E, U>,
    C: GraphChildExec<E, U>,
{
    pub node: N,
    pub children: C,
    _phantom: core::marker::PhantomData<(E, U)>,
}

impl<N, C, E, U> GraphNodeWrapper<N, C, E, U>
where
    N: GraphNodeExec<E, U>,
    C: GraphChildExec<E, U>,
{
    pub fn new(node: N, children: C) -> Self {
        Self {
            node,
            children,
            _phantom: Default::default(),
        }
    }
}

impl<N, C, E, U> GraphNode<E, U> for GraphNodeWrapper<N, C, E, U>
where
    N: GraphNodeExec<E, U>,
    C: GraphChildExec<E, U>,
{
    fn node_exec(&self, context: &mut dyn EventEvalContext<E>, user_data: &mut U) {
        self.node.graph_exec(context, &self.children, user_data)
    }
}
