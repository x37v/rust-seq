use crate::{
    event::EventEvalContext,
    graph::{GraphChildExec, GraphNodeExec},
    param::{ParamGet, ParamSet},
};

///A graph node that stores a value and then calls its children.
pub struct ParamStore<T, G, S>
where
    G: ParamGet<T>,
    S: ParamSet<T>,
{
    get: G,
    set: S,
    _phantom: core::marker::PhantomData<T>,
}

impl<T, G, S> ParamStore<T, G, S>
where
    G: ParamGet<T>,
    S: ParamSet<T>,
{
    pub fn new(get: G, set: S) -> Self {
        Self {
            get,
            set,
            _phantom: Default::default(),
        }
    }
}

impl<T, G, S, E> GraphNodeExec<E> for ParamStore<T, G, S>
where
    G: ParamGet<T>,
    S: ParamSet<T>,
{
    fn graph_exec(&self, context: &mut dyn EventEvalContext<E>, children: &dyn GraphChildExec<E>) {
        self.set.set(self.get.get());
        children.child_exec_all(context);
    }
}
