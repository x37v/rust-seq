use crate::binding::ParamBindingGet;
use crate::ptr::{SShrPtr, ShrPtr};
use crate::trigger::*;

/// A trigger that evaluates its children only when the associated gate value is true
pub struct TriggerGate<T> {
    trigger_index: TriggerId,
    gate: ShrPtr<T>,
    children: Vec<SShrPtr<dyn Trigger>>,
}

impl<T> TriggerGate<T>
where
    T: ParamBindingGet<bool>,
{
    /// Construct a new `TriggerGate`
    ///
    /// # Arguments
    ///
    /// * `gate` - the binding value for the gate
    /// * `children` - a list of children to trigger
    pub fn new(gate: ShrPtr<T>, children: Vec<SShrPtr<dyn Trigger>>) -> Self {
        Self {
            trigger_index: TriggerId::new(),
            gate,
            children,
        }
    }
}

impl<T> Trigger for TriggerGate<T>
where
    T: ParamBindingGet<bool>,
{
    fn trigger_index(&self) -> TriggerId {
        self.trigger_index
    }

    fn trigger_eval(&self, tick: usize, context: &mut dyn ScheduleTrigger) {
        if self.gate.get() {
            for t in self.children.iter() {
                t.lock().trigger_eval(tick, context);
            }
        }
    }
}
