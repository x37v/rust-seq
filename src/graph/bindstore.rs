use crate::binding::{ParamBindingGet, ParamBindingSet};
use crate::event::EventEvalContext;
use crate::graph::{GraphChildExec, GraphIndexExec, GraphNodeExec};
use core::marker::PhantomData;

/// A graph node that stores a bound get into a set and then calls all of its children.
pub struct BindStoreNode<T, G, S>
where
    T: Send + Copy,
    G: ParamBindingGet<T>,
    S: ParamBindingSet<T>,
{
    get: G,
    set: S,
    phantom: PhantomData<T>,
}

/// A graph index child that stores the index that it is called with.
pub struct BindStoreIndexChild<S>
where
    S: ParamBindingSet<usize>,
{
    set: S,
}

impl<T, G, S> BindStoreNode<T, G, S>
where
    T: Send + Copy,
    G: ParamBindingGet<T>,
    S: ParamBindingSet<T>,
{
    pub fn new(get: G, set: S) -> Self {
        Self {
            get,
            set,
            phantom: PhantomData,
        }
    }
}

impl<S> BindStoreIndexChild<S>
where
    S: ParamBindingSet<usize>,
{
    pub fn new(set: S) -> Self {
        Self { set }
    }
}

impl<T, G, S> GraphNodeExec for BindStoreNode<T, G, S>
where
    T: Send + Copy,
    G: ParamBindingGet<T>,
    S: ParamBindingSet<T>,
{
    fn graph_exec(
        &mut self,
        context: &mut dyn EventEvalContext,
        children: &mut dyn GraphChildExec,
    ) {
        self.set.set(self.get.get());
        children.child_exec_all(context);
    }
}

impl<S> GraphIndexExec for BindStoreIndexChild<S>
where
    S: ParamBindingSet<usize>,
{
    fn exec_index(&mut self, index: usize, _context: &mut dyn EventEvalContext) {
        self.set.set(index);
    }
}
