use crate::event::*;
use crate::graph::{GraphChildExec, GraphNode, GraphNodeContainer, GraphNodeExec};

pub struct GraphNodeWrapper<E, C>
where
    E: GraphNodeExec,
    C: GraphChildExec,
{
    pub exec: E,
    pub children: C,
}

impl<E, C> GraphNodeWrapper<E, C>
where
    E: GraphNodeExec,
    C: GraphChildExec,
{
    pub fn new(exec: E, children: C) -> Self {
        Self { exec, children }
    }
}

impl<E, C> core::convert::Into<GraphNodeContainer> for GraphNodeWrapper<E, C>
where
    E: GraphNodeExec + 'static,
    C: GraphChildExec + 'static,
{
    fn into(self) -> GraphNodeContainer {
        GraphNodeContainer::new(self)
    }
}

impl<E, C> GraphNode for GraphNodeWrapper<E, C>
where
    E: GraphNodeExec,
    C: GraphChildExec,
{
    fn node_exec(&self, context: &mut dyn EventEvalContext) {
        self.exec.graph_exec(context, &self.children)
    }
}
