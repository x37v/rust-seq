use crate::binding::{ParamBindingGet, ParamBindingSet};
use crate::event::EventEvalContext;
use crate::graph::{GraphChildExec, GraphNodeExec};
use core::marker::PhantomData;

/// A graph node that stores a bound get into a set and then calls all of its children.
pub struct BindStore<T, G, S>
where
    T: Send + Copy,
    G: ParamBindingGet<T>,
    S: ParamBindingSet<T>,
{
    get: G,
    set: S,
    phantom: PhantomData<T>,
}

impl<T, G, S> BindStore<T, G, S>
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

impl<T, G, S> GraphNodeExec for BindStore<T, G, S>
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
