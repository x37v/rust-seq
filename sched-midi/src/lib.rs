extern crate sched;
use sched::binding::{ParamBindingGet, SpinlockParamBinding, SpinlockParamBindingP, ValueSet};
use sched::ScheduleTrigger;
use sched::TimeResched;
use sched::TimeSched;
use std::sync::mpsc::SyncSender;
use std::sync::Arc;

pub enum MidiValue {
    Note {
        on: bool,
        chan: u8,
        num: u8,
        vel: u8,
    },
}

pub struct MidiValueAt {
    value: MidiValue,
    tick: usize,
}

impl MidiValueAt {
    pub fn new(tick: usize, value: MidiValue) -> Self {
        Self { value, tick }
    }

    pub fn tick(&self) -> usize {
        self.tick
    }

    pub fn value(&self) -> &MidiValue {
        &self.value
    }
}

pub struct NoteTrigger {
    trigger_index: usize,
    chan: SpinlockParamBindingP<u8>,
    on: SpinlockParamBindingP<bool>,
    num: SpinlockParamBindingP<u8>,
    vel: SpinlockParamBindingP<u8>,
    sender: SyncSender<MidiValueAt>,
}

impl NoteTrigger {
    pub fn new(trigger_index: usize, sender: SyncSender<MidiValueAt>) -> Self {
        Self {
            trigger_index,
            chan: Arc::new(SpinlockParamBinding::new(0)),
            on: Arc::new(SpinlockParamBinding::new(false)),
            num: Arc::new(SpinlockParamBinding::new(0)),
            vel: Arc::new(SpinlockParamBinding::new(0)),
            sender,
        }
    }

    pub fn trigger_index(&self) -> usize {
        self.trigger_index
    }

    pub fn eval(&self, tick: usize) {
        let v = MidiValueAt::new(
            tick,
            MidiValue::Note {
                on: self.on.get(),
                chan: self.chan.get(),
                num: self.num.get(),
                vel: self.vel.get(),
            },
        );
        if let Err(e) = self.sender.try_send(v) {
            println!("midi send error: {:?}", e);
        }
    }

    pub fn note(
        &self,
        time: TimeSched,
        schedule: &mut dyn ScheduleTrigger,
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

    pub fn note_with_dur(
        &self,
        on_time: TimeSched,
        dur: TimeResched,
        schedule: &mut dyn ScheduleTrigger,
        chan: u8,
        num: u8,
        vel: u8,
    ) {
        let off_time = schedule.add_time(&on_time, &dur);
        self.note(on_time, schedule, chan, true, num, vel);
        self.note(off_time, schedule, chan, false, num, vel);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
