use crate::binding::{ParamBindingGet, ParamBindingSet};
use crate::event::{EventEval, EventEvalContext};
use crate::tick::TickResched;
use core::marker::PhantomData;

/// An event that stores a value from a get into a set.
pub struct BindStoreEvent<T, G, S>
where
    T: Send + Copy,
    G: ParamBindingGet<T>,
    S: ParamBindingSet<T>,
{
    get: G,
    set: S,
    phantom: PhantomData<T>,
}

impl<T, G, S> BindStoreEvent<T, G, S>
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

impl<T, G, S> EventEval for BindStoreEvent<T, G, S>
where
    T: Send + Copy,
    G: ParamBindingGet<T>,
    S: ParamBindingSet<T>,
{
    fn event_eval(&mut self, _context: &mut dyn EventEvalContext) -> TickResched {
        self.set.set(self.get.get());
        TickResched::None
    }
}
