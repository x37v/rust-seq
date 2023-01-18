use crate::{
    event::EventEvalContext,
    graph::{GraphChildExec, GraphNodeExec},
    param::ParamGet,
};

///A graph node that calls its children only when its gate parameter is true
pub struct Gate<P>
where
    P: ParamGet<bool>,
{
    gate: P,
}

impl<P> Gate<P>
where
    P: ParamGet<bool>,
{
    /// Create a new gate with the given parameter.
    pub fn new(gate: P) -> Self {
        Self { gate }
    }
}

impl<P, E> GraphNodeExec<E> for Gate<P>
where
    P: ParamGet<bool>,
{
    fn graph_exec(&self, context: &mut dyn EventEvalContext<E>, children: &dyn GraphChildExec<E>) {
        if self.gate.get() {
            children.child_exec_all(context);
        }
    }
}
