use crate::{
    event::EventEvalContext,
    graph::{GraphChildExec, GraphNodeExec},
    param::ParamSet,
};

/// Stores the context tick
pub struct TickStore<T, U> {
    storage: T,
    _phantom: core::marker::PhantomData<U>,
}

impl<T, U> TickStore<T, U>
where
    T: ParamSet<usize, U>,
{
    pub fn new(storage: T) -> Self {
        Self {
            storage,
            _phantom: Default::default(),
        }
    }
}

impl<T, E, U> GraphNodeExec<E, U> for TickStore<T, U>
where
    T: ParamSet<usize, U>,
{
    fn graph_exec(
        &self,
        context: &mut dyn EventEvalContext<E>,
        children: &dyn GraphChildExec<E, U>,
        user_data: &mut U,
    ) {
        self.storage.set(context.context_tick_now(), user_data);
        children.child_exec_all(context, user_data);
    }
}
