extern crate sched;
use sched::binding::{ParamBindingGet, SpinlockParamBinding, SpinlockParamBindingP, ValueSet};
use sched::ScheduleTrigger;
use sched::TimeResched;
use sched::TimeSched;
use std::sync::mpsc::SyncSender;
use std::sync::Arc;

#[derive(Clone, Copy, Debug)]
pub enum MidiValue {
    Note {
        on: bool,
        chan: u8,
        num: u8,
        vel: u8,
    },
}

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum MidiStatus {
    NoteOn = 0x90,
    NoteOff = 0x80,
    AfterTouch = 0xA0,
    ContCtrl = 0xB0,
    ProgChng = 0xC0,
    ChanPres = 0xD0,
    PitchBend = 0xE0,

    Clock = 0xF8,
    Tick = 0xF9,
    Start = 0xFA,
    Continue = 0xFB,
    Stop = 0xFC,
    ActiveSense = 0xFE,
    Reset = 0xFF,

    TcQFrame = 0xF1,
    SongPos = 0xF2,
    SongSel = 0xF3,
    TuneReq = 0xF6,

    SysexBeg = 0xF0,
    SysexEnd = 0xF7,
}

impl From<MidiStatus> for u8 {
    fn from(v: MidiStatus) -> u8 {
        v as u8
    }
}

pub struct MidiValueAt {
    value: MidiValue,
    tick: usize,
}

pub struct MidiValueIterator<'a> {
    value: &'a MidiValue,
    index: u8,
}

impl MidiValue {
    pub fn iter(&self) -> MidiValueIterator {
        MidiValueIterator {
            value: self,
            index: 0,
        }
    }
}

impl<'a> Iterator for MidiValueIterator<'a> {
    type Item = u8;

    fn next(&mut self) -> Option<u8> {
        let r = match self.value {
            MidiValue::Note { on, chan, num, vel } => match self.index {
                0 => Some(
                    chan & 0x0F | if *on {
                        MidiStatus::NoteOn
                    } else {
                        MidiStatus::NoteOff
                    } as u8,
                ),
                1 => Some(num & 0x7F),
                2 => Some(vel & 0x7F),
                _ => None,
            },
        };
        //so we never overflow
        if r.is_some() {
            self.index += 1;
        }
        r
    }
}

impl<'a> ExactSizeIterator for MidiValueIterator<'a> {
    fn len(&self) -> usize {
        match self.value {
            MidiValue::Note { .. } => 3,
        }
    }
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
