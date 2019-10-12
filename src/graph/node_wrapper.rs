use crate::event::*;
use crate::graph::{GraphChildExec, GraphNode, GraphNodeExec};

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

impl<E, C> GraphNode for GraphNodeWrapper<E, C>
where
    E: GraphNodeExec,
    C: GraphChildExec,
{
    fn node_exec(&mut self, context: &mut dyn EventEvalContext) {
        self.exec.graph_exec(context, &mut self.children)
    }
}
