use crate::{
    event::EventEvalContext,
    graph::{GraphChildExec, GraphNodeExec},
    param::{ParamGet, ParamSet},
};

///A graph node that stores a value and then calls its children.
pub struct ParamStore<T, G, S, U>
where
    G: ParamGet<T, U>,
    S: ParamSet<T, U>,
{
    get: G,
    set: S,
    _phantom: core::marker::PhantomData<(T, U)>,
}

impl<T, G, S, U> ParamStore<T, G, S, U>
where
    G: ParamGet<T, U>,
    S: ParamSet<T, U>,
{
    pub fn new(get: G, set: S) -> Self {
        Self {
            get,
            set,
            _phantom: Default::default(),
        }
    }
}

impl<T, G, S, E, U> GraphNodeExec<E, U> for ParamStore<T, G, S, U>
where
    T: Send,
    U: Send,
    G: ParamGet<T, U>,
    S: ParamSet<T, U>,
{
    fn graph_exec(
        &self,
        context: &mut dyn EventEvalContext<E>,
        children: &dyn GraphChildExec<E, U>,
        user_data: &mut U,
    ) {
        self.set.set(self.get.get(user_data), user_data);
        children.child_exec_all(context, user_data);
    }
}
