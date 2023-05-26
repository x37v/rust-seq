use crate::{
    event::EventEvalContext,
    graph::{GraphChildExec, GraphNodeExec},
    param::{ParamGet, ParamSet},
};

///A graph node that re-triggers a given number of times, storing its value before calling its
///children.
pub struct Repeat<T, G, S, U>
where
    T: num_traits::PrimInt + num_traits::sign::Unsigned,
    G: ParamGet<T, U>,
    S: ParamSet<T, U>,
{
    repeats: G,
    index: S,
    _phantom: core::marker::PhantomData<(T, U)>,
}

impl<T, G, S, U> Repeat<T, G, S, U>
where
    T: num_traits::PrimInt + num_traits::sign::Unsigned,
    G: ParamGet<T, U>,
    S: ParamSet<T, U>,
{
    pub fn new(repeats: G, index: S) -> Self {
        Self {
            repeats,
            index,
            _phantom: Default::default(),
        }
    }
}

impl<T, G, S, E, U> GraphNodeExec<E, U> for Repeat<T, G, S, U>
where
    T: num_traits::PrimInt
        + num_traits::sign::Unsigned
        + core::ops::Add<T, Output = T>
        + PartialOrd
        + Clone
        + num_traits::One
        + Send,
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
        let r = self.repeats.get(user_data);
        for i in num_iter::range(T::zero(), r) {
            self.index.set(i, user_data);
            children.child_exec_all(context, user_data);
        }
    }
}
