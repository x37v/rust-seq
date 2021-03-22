use crate::event::EventEvalContext;
use crate::graph::{GraphChildExec, GraphLeafExec, GraphNodeExec};

pub struct LeafFunc<F, E> {
    func: F,
    _phantom: core::marker::PhantomData<E>,
}

pub struct NodeFunc<F, E> {
    func: F,
    _phantom: core::marker::PhantomData<E>,
}

impl<F, E> LeafFunc<F, E>
where
    F: Fn(&mut dyn EventEvalContext<E>) + Send,
    E: Send,
{
    pub fn new(func: F) -> Self {
        Self {
            func,
            _phantom: Default::default(),
        }
    }
}

impl<F, E> GraphLeafExec<E> for LeafFunc<F, E>
where
    F: Fn(&mut dyn EventEvalContext<E>) + Send,
    E: Send,
{
    fn graph_exec(&self, context: &mut dyn EventEvalContext<E>) {
        (self.func)(context);
    }
}

impl<F, E> NodeFunc<F, E>
where
    F: Fn(&mut dyn EventEvalContext<E>, &dyn GraphChildExec<E>) + Send,
    E: Send,
{
    pub fn new(func: F) -> Self {
        Self {
            func,
            _phantom: Default::default(),
        }
    }
}

impl<F, E> GraphNodeExec<E> for NodeFunc<F, E>
where
    F: Fn(&mut dyn EventEvalContext<E>, &dyn GraphChildExec<E>) + Send,
    E: Send,
{
    fn graph_exec(&self, context: &mut dyn EventEvalContext<E>, children: &dyn GraphChildExec<E>) {
        (self.func)(context, children);
    }
}
