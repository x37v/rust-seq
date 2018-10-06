extern crate sched;
use sched::binding::{SpinlockParamBinding, SpinlockParamBindingP, ValueSet};
use sched::ScheduleTrigger;
use sched::TimeSched;
use std::sync::Arc;

pub struct NoteTrigger {
    trigger_index: usize,
    chan: SpinlockParamBindingP<u8>,
    on: SpinlockParamBindingP<bool>,
    num: SpinlockParamBindingP<u8>,
    vel: SpinlockParamBindingP<u8>,
}

impl NoteTrigger {
    pub fn new(trigger_index: usize) -> Self {
        Self {
            trigger_index,
            chan: Arc::new(SpinlockParamBinding::new(0)),
            on: Arc::new(SpinlockParamBinding::new(false)),
            num: Arc::new(SpinlockParamBinding::new(0)),
            vel: Arc::new(SpinlockParamBinding::new(0)),
        }
    }

    pub fn note(
        &self,
        time: TimeSched,
        schedule: &mut impl ScheduleTrigger,
        chan: u8,
        on: bool,
        num: u8,
        vel: u8,
    ) {
        schedule.schedule_valued_trigger(
            time,
            self.trigger_index,
            &[
                ValueSet::U8(chan, self.chan.clone()),
                ValueSet::U8(num, self.num.clone()),
                ValueSet::U8(vel, self.vel.clone()),
                ValueSet::BOOL(on, self.on.clone()),
            ],
        );
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
