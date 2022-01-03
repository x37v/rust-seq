use crate::{
    event::EventEvalContext,
    graph::{GraphChildExec, GraphNodeExec},
    param::ParamSet,
};

/// Stores the context tick
pub struct TickStore<T> {
    storage: T,
}

impl<T> TickStore<T>
where
    T: ParamSet<usize>,
{
    pub fn new(storage: T) -> Self {
        Self { storage }
    }
}

impl<T, E> GraphNodeExec<E> for TickStore<T>
where
    T: ParamSet<usize>,
    E: Send,
{
    fn graph_exec(&self, context: &mut dyn EventEvalContext<E>, children: &dyn GraphChildExec<E>) {
        self.storage.set(context.context_tick_now());
        children.child_exec_all(context);
    }
}
