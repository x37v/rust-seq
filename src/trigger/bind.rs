use binding::ParamBindingGet;
use binding::ParamBindingSet;
use ptr::{SShrPtr, ShrPtr};
use std::marker::PhantomData;
use trigger::*;

/// A trigger binds a value
pub struct TriggerBind<T, In, Out> {
    trigger_index: TriggerId,
    in_binding: ShrPtr<In>,
    out_binding: ShrPtr<Out>,
    _phantom: PhantomData<fn() -> T>,
}

impl<T, In, Out> TriggerBind<T, In, Out>
where
    In: ParamBindingGet<T>,
    Out: ParamBindingSet<T>,
{
    /// Construct a new `Bind`
    ///
    /// # Arguments
    ///
    /// * `in_binding` - the input to read from
    /// * `out_binding` - the output to write to
    pub fn new(in_binding: ShrPtr<In>, out_binding: ShrPtr<Out>) -> Self {
        Self {
            trigger_index: TriggerId::new(),
            in_binding,
            out_binding,
            _phantom: Default::default(),
        }
    }
}

impl<T, In, Out> Trigger for TriggerBind<T, In, Out>
where
    In: ParamBindingGet<T>,
    Out: ParamBindingSet<T>,
{
    fn trigger_index(&self) -> TriggerId {
        self.trigger_index
    }

    fn trigger_eval(&self, _tick: usize, _context: &mut dyn ScheduleTrigger) {
        self.out_binding.set(self.in_binding.get());
    }
}
