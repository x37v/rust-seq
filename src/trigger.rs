use std::sync::atomic::{AtomicUsize, Ordering};
use ScheduleTrigger;

static ID_COUNT: AtomicUsize = AtomicUsize::new(0);

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct TriggerId {
    id: usize,
}

pub trait Trigger {
    fn trigger_index(&self) -> TriggerId;
    fn trigger_eval(&self, tick: usize, context: &mut dyn ScheduleTrigger);
}

impl TriggerId {
    pub fn new() -> Self {
        Self {
            id: ID_COUNT.fetch_add(1, Ordering::Relaxed),
        }
    }
}
