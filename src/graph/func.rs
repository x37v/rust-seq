use crate::event::EventEvalContext;
use crate::graph::{GraphChildExec, GraphLeafExec, GraphNodeExec};

pub struct LeafFunc<F, E, U> {
    func: F,
    _phantom: core::marker::PhantomData<(E, U)>,
}

pub struct NodeFunc<F, E, U> {
    func: F,
    _phantom: core::marker::PhantomData<(E, U)>,
}

impl<F, E, U> LeafFunc<F, E, U>
where
    F: Fn(&mut dyn EventEvalContext<E>),
{
    pub fn new(func: F) -> Self {
        Self {
            func,
            _phantom: Default::default(),
        }
    }
}

impl<F, E, U> GraphLeafExec<E, U> for LeafFunc<F, E, U>
where
    E: Send,
    U: Send,
    F: Fn(&mut dyn EventEvalContext<E>, &mut U) + Send,
{
    fn graph_exec(&self, context: &mut dyn EventEvalContext<E>, user_data: &mut U) {
        (self.func)(context, user_data);
    }
}

impl<F, E, U> NodeFunc<F, E, U>
where
    F: Fn(&mut dyn EventEvalContext<E>, &dyn GraphChildExec<E, U>, &mut U),
{
    pub fn new(func: F) -> Self {
        Self {
            func,
            _phantom: Default::default(),
        }
    }
}

impl<F, E, U> GraphNodeExec<E, U> for NodeFunc<F, E, U>
where
    E: Send,
    U: Send,
    F: Fn(&mut dyn EventEvalContext<E>, &dyn GraphChildExec<E, U>, &mut U) + Send,
{
    fn graph_exec(
        &self,
        context: &mut dyn EventEvalContext<E>,
        children: &dyn GraphChildExec<E, U>,
        user_data: &mut U,
    ) {
        (self.func)(context, children, user_data);
    }
}
