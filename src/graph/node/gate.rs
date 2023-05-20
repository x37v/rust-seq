use crate::{
    event::EventEvalContext,
    graph::{GraphChildExec, GraphNodeExec},
    param::ParamGet,
};

///A graph node that calls its children only when its gate parameter is true
pub struct Gate<P, U>
where
    P: ParamGet<bool, U>,
{
    gate: P,
    _phantom: core::marker::PhantomData<U>,
}

impl<P, U> Gate<P, U>
where
    P: ParamGet<bool, U>,
{
    /// Create a new gate with the given parameter.
    pub fn new(gate: P) -> Self {
        Self {
            gate,
            _phantom: Default::default(),
        }
    }
}

impl<P, E, U> GraphNodeExec<E, U> for Gate<P, U>
where
    P: ParamGet<bool, U>,
{
    fn graph_exec(
        &self,
        context: &mut dyn EventEvalContext<E>,
        children: &dyn GraphChildExec<E, U>,
        user_data: &mut U,
    ) {
        if self.gate.get(user_data) {
            children.child_exec_all(context, user_data);
        }
    }
}
