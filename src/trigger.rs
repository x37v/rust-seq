use crate::binding::set::BindingSet;
use crate::time::{TimeResched, TimeSched};
use std::sync::atomic::{AtomicUsize, Ordering};

pub mod bind;

#[cfg(feature = "std")]
pub mod gate;

pub trait Trigger {
    fn trigger_index(&self) -> TriggerId;
    fn trigger_eval(&self, tick: usize, context: &mut dyn ScheduleTrigger);
}

pub trait ScheduleTrigger {
    fn schedule_trigger(&mut self, time: TimeSched, index: TriggerId);
    fn schedule_valued_trigger(&mut self, time: TimeSched, index: TriggerId, values: &[BindingSet]);
    fn schedule_value(&mut self, time: TimeSched, value: &BindingSet);
    fn add_time(&self, time: &TimeSched, dur: &TimeResched) -> TimeSched;
}

static ID_COUNT: AtomicUsize = AtomicUsize::new(0);

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct TriggerId {
    id: usize,
}

impl TriggerId {
    pub fn new() -> Self {
        Self {
            id: ID_COUNT.fetch_add(1, Ordering::Relaxed),
        }
    }
}

impl Default for TriggerId {
    fn default() -> Self {
        Self::new()
    }
}
