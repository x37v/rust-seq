use crate::binding::ParamBindingGet;
use crate::ptr::SShrPtr;
use crate::trigger::*;

/// A trigger that evaluates its children only when the associated gate value is true
pub struct TriggerGate<Gate, Children>
where
    Gate: ParamBindingGet<bool>,
    for<'a> &'a Children: core::iter::IntoIterator<Item = &'a SShrPtr<dyn Trigger>>,
{
    trigger_index: TriggerId,
    gate: Gate,
    children: Children,
}

impl<Gate, Children> TriggerGate<Gate, Children>
where
    Gate: ParamBindingGet<bool>,
    for<'a> &'a Children: core::iter::IntoIterator<Item = &'a SShrPtr<dyn Trigger>>,
{
    /// Construct a new `TriggerGate`
    ///
    /// # Arguments
    ///
    /// * `gate` - the binding value for the gate
    /// * `children` - a list of children to trigger
    pub fn new(gate: Gate, children: Children) -> Self {
        Self {
            trigger_index: TriggerId::new(),
            gate,
            children,
        }
    }
}

impl<Gate, Children> Trigger for TriggerGate<Gate, Children>
where
    Gate: ParamBindingGet<bool>,
    for<'a> &'a Children: core::iter::IntoIterator<Item = &'a SShrPtr<dyn Trigger>>,
{
    fn trigger_index(&self) -> TriggerId {
        self.trigger_index
    }

    fn trigger_eval(&self, tick: usize, context: &mut dyn ScheduleTrigger) {
        if self.gate.get() {
            for t in self.children.into_iter() {
                t.lock().trigger_eval(tick, context);
            }
        }
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;

    #[test]
    fn clamp() {
        let g = false;
        let v: Vec<SShrPtr<dyn Trigger>> = Vec::new();

        let _ = TriggerGate::new(g, v);
    }
}
