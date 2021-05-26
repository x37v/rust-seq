use crate::{
    event::EventEvalContext,
    graph::{GraphChildExec, GraphNodeExec},
    param::{ParamGet, ParamSet},
};

///A graph node that re-triggers a given number of times, storing its value before calling its
///children.
pub struct Repeat<T, G, S>
where
    T: Send + num_traits::PrimInt + num_traits::sign::Unsigned,
    G: ParamGet<T>,
    S: ParamSet<T>,
{
    repeats: G,
    index: S,
    _phantom: core::marker::PhantomData<T>,
}

impl<T, G, S> Repeat<T, G, S>
where
    T: Send + num_traits::PrimInt + num_traits::sign::Unsigned,
    G: ParamGet<T>,
    S: ParamSet<T>,
{
    pub fn new(repeats: G, index: S) -> Self {
        Self {
            repeats,
            index,
            _phantom: Default::default(),
        }
    }
}

impl<T, G, S, E> GraphNodeExec<E> for Repeat<T, G, S>
where
    T: Send
        + num_traits::PrimInt
        + num_traits::sign::Unsigned
        + core::ops::Add<T, Output = T>
        + PartialOrd
        + Clone
        + num_traits::One,
    G: ParamGet<T>,
    S: ParamSet<T>,
    E: Send,
{
    fn graph_exec(&self, context: &mut dyn EventEvalContext<E>, children: &dyn GraphChildExec<E>) {
        let r = self.repeats.get();
        for i in num_iter::range(T::zero(), r) {
            self.index.set(i);
            children.child_exec_all(context);
        }
    }
}
