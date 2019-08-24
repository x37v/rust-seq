use crate::binding::ParamBindingGet;
use crate::binding::ParamBindingSet;
use crate::trigger::*;
use core::marker::PhantomData;

/// A trigger binds a value
pub struct TriggerBind<'a, T: 'a, In: 'a, Out: 'a> {
    trigger_index: TriggerId,
    in_binding: In,
    out_binding: Out,
    _phantom: PhantomData<fn() -> &'a T>,
}

impl<'a, T, In, Out> TriggerBind<'a, T, In, Out>
where
    T: 'a,
    In: ParamBindingGet<T>,
    Out: ParamBindingSet<T>,
{
    /// Construct a new `Bind`
    ///
    /// # Arguments
    ///
    /// * `in_binding` - the input to read from
    /// * `out_binding` - the output to write to
    pub fn new(in_binding: In, out_binding: Out) -> Self {
        Self {
            trigger_index: TriggerId::new(),
            in_binding,
            out_binding,
            _phantom: Default::default(),
        }
    }
}

impl<'a, T, In, Out> Trigger for TriggerBind<'a, T, In, Out>
where
    T: 'a,
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
