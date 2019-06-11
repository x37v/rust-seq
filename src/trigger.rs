use crate::binding::set::BindingSet;
use crate::binding::ParamBindingLatch;
use crate::list::LinkedList;
use crate::time::{TimeResched, TimeSched};
use core::sync::atomic::{AtomicUsize, Ordering};

pub mod bind;
pub mod gate;

pub trait Trigger {
    fn trigger_index(&self) -> TriggerId;
    fn trigger_eval(&self, tick: usize, context: &mut dyn ScheduleTrigger);
}

pub trait TrigCall: Send {
    fn set_index(&mut self, index: Option<TriggerId>);
    fn index(&self) -> Option<TriggerId>;
    fn add_value(&mut self, binding: BindingSet);
    fn latch_values(&mut self);
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

pub struct LListTrigCall<VL>
where
    VL: LinkedList<BindingSet>,
    for<'a> &'a VL: core::iter::IntoIterator<Item = &'a mut BindingSet>,
{
    index: Option<TriggerId>,
    values: VL,
}

impl<VL> TrigCall for LListTrigCall<VL>
where
    VL: LinkedList<BindingSet>,
    for<'a> &'a VL: core::iter::IntoIterator<Item = &'a mut BindingSet>,
{
    fn set_index(&mut self, index: Option<TriggerId>) {
        self.index = index;
    }
    fn index(&self) -> Option<TriggerId> {
        self.index
    }
    fn add_value(&mut self, binding: BindingSet) {
        self.values.push_back(binding);
    }
    fn latch_values(&mut self) {
        for v in &self.values {
            v.store()
        }
    }
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
