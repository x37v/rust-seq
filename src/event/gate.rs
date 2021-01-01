use crate::{
    binding::ParamBindingGet,
    event::{EventEval, EventEvalContext},
    tick::TickResched,
};
use std::sync::Arc;

pub type ArcMutexEvent = Arc<crate::mutex::Mutex<dyn EventEval>>;

/// An event that owns another and executes it until
/// either the owned event doesn't reschedule or the gate
/// binding resolves to false.
pub struct GateEvent<G>
where
    G: ParamBindingGet<bool>,
{
    gate: G,
    event: ArcMutexEvent,
}

impl<G> GateEvent<G>
where
    G: ParamBindingGet<bool>,
{
    /// Create a new gate event.
    pub fn new(gate: G, event: ArcMutexEvent) -> Self {
        Self { gate, event }
    }
}

impl<G> EventEval for GateEvent<G>
where
    G: ParamBindingGet<bool>,
{
    fn event_eval(&mut self, context: &mut dyn EventEvalContext) -> TickResched {
        if !self.gate.get() {
            TickResched::None
        } else {
            self.event.lock().event_eval(context)
        }
    }
}
