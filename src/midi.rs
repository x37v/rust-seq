use crate::base::{TimeResched, TimeSched};
use crate::binding::{set::BindingSet, spinlock::SpinlockParamBinding, ParamBindingGet};
use crate::ptr::ShrPtr;
use std::sync::mpsc::SyncSender;
use crate::trigger::{ScheduleTrigger, Trigger, TriggerId};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MidiValue {
    None,
    Note {
        on: bool,
        chan: u8,
        num: u8,
        vel: u8,
    },
    ContCtrl {
        chan: u8,
        num: u8,
        val: u8,
    },
}

trait MidiClamp {
    fn mclamp(&self) -> u8;
}

impl MidiClamp for u8 {
    fn mclamp(&self) -> u8 {
        num::clamp(*self, 0, 127)
    }
}

fn status_byte(status: MidiStatus, chan: u8) -> u8 {
    chan & 0x0F | status as u8
}

#[derive(Clone, Copy, Debug, PartialEq)]
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

    pub fn try_from(bytes: &[u8]) -> Option<Self> {
        match bytes.len() {
            3 => {
                let chan = bytes[0] & 0x0F;
                let status = bytes[0] & 0xF0;
                if status == MidiStatus::NoteOn as u8 {
                    Some(MidiValue::Note {
                        on: true,
                        chan,
                        num: bytes[1],
                        vel: bytes[2],
                    })
                } else if status == MidiStatus::NoteOff as u8 {
                    Some(MidiValue::Note {
                        on: false,
                        chan,
                        num: bytes[1],
                        vel: bytes[2],
                    })
                } else if status == MidiStatus::ContCtrl as u8 {
                    Some(MidiValue::ContCtrl {
                        chan,
                        num: bytes[1],
                        val: bytes[2],
                    })
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

impl<'a> Iterator for MidiValueIterator<'a> {
    type Item = u8;

    fn next(&mut self) -> Option<u8> {
        let r = match self.value {
            MidiValue::Note { on, chan, num, vel } => match self.index {
                0 => Some(status_byte(
                    if *on {
                        MidiStatus::NoteOn
                    } else {
                        MidiStatus::NoteOff
                    },
                    *chan,
                )),
                1 => Some(num.mclamp()),
                2 => Some(vel.mclamp()),
                _ => None,
            },
            MidiValue::ContCtrl { chan, num, val } => match self.index {
                0 => Some(status_byte(MidiStatus::ContCtrl, *chan)),
                1 => Some(num.mclamp()),
                2 => Some(val.mclamp()),
                _ => None,
            },
            MidiValue::None => None,
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
            MidiValue::ContCtrl { .. } => 3,
            MidiValue::None => 0,
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

pub struct MidiTrigger {
    trigger_index: TriggerId,
    value: ShrPtr<SpinlockParamBinding<MidiValue>>,
    sender: SyncSender<MidiValueAt>,
}

impl MidiTrigger {
    pub fn new(sender: SyncSender<MidiValueAt>) -> Self {
        Self {
            trigger_index: TriggerId::new(),
            value: new_shrptr!(SpinlockParamBinding::new(MidiValue::None)),
            sender,
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
        self.add(schedule, time, MidiValue::Note { on, chan, num, vel });
    }

    pub fn note_with_dur(
        &self,
        on_time: TimeSched,
        dur: TimeResched,
        schedule: &mut dyn ScheduleTrigger,
        chan: u8,
        num: u8,
        on_vel: u8,
        off_vel: u8,
    ) {
        let off_time = schedule.add_time(&on_time, &dur);
        self.note(on_time, schedule, chan, true, num, on_vel);
        self.note(off_time, schedule, chan, false, num, off_vel);
    }

    pub fn cont_ctrl(
        &self,
        time: TimeSched,
        schedule: &mut dyn ScheduleTrigger,
        chan: u8,
        num: u8,
        val: u8,
    ) {
        self.add(schedule, time, MidiValue::ContCtrl { chan, num, val });
    }

    pub fn add(&self, schedule: &mut dyn ScheduleTrigger, time: TimeSched, value: MidiValue) {
        schedule.schedule_valued_trigger(
            time,
            self.trigger_index,
            &[BindingSet::Midi(value, self.value.clone())],
        );
    }
}

impl Trigger for MidiTrigger {
    fn trigger_index(&self) -> TriggerId {
        self.trigger_index
    }

    fn trigger_eval(&self, tick: usize, _context: &mut dyn ScheduleTrigger) {
        let msg = self.value.get();
        match msg {
            MidiValue::None => (),
            _ => {
                let v = MidiValueAt::new(tick, msg);
                if let Err(e) = self.sender.try_send(v) {
                    println!("midi send error: {:?}", e);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_from() {
        //NoteOn = 0x90,
        //NoteOff = 0x80,
        //ContCtrl = 0xB0,

        //too short
        assert_eq!(None, MidiValue::try_from(&[1]));
        assert_eq!(None, MidiValue::try_from(&[7]));
        assert_eq!(None, MidiValue::try_from(&[0x90, 1]));
        assert_eq!(None, MidiValue::try_from(&[0x80, 1]));
        assert_eq!(None, MidiValue::try_from(&[0x91, 1]));
        assert_eq!(None, MidiValue::try_from(&[0x81, 1]));
        assert_eq!(None, MidiValue::try_from(&[0xB1, 1]));
        assert_eq!(None, MidiValue::try_from(&[0xB0, 1]));

        //just right
        assert_eq!(
            Some(MidiValue::Note {
                on: false,
                chan: 3,
                num: 2,
                vel: 64
            }),
            MidiValue::try_from(&[0x83, 2, 64])
        );
        assert_eq!(
            Some(MidiValue::Note {
                on: false,
                chan: 2,
                num: 7,
                vel: 96
            }),
            MidiValue::try_from(&[0x82, 7, 96])
        );

        assert_eq!(
            Some(MidiValue::ContCtrl {
                chan: 2,
                num: 7,
                val: 96
            }),
            MidiValue::try_from(&[0xB2, 7, 96])
        );

        assert_eq!(
            Some(MidiValue::ContCtrl {
                chan: 15,
                num: 123,
                val: 15
            }),
            MidiValue::try_from(&[0xBF, 123, 15])
        );

        //too long
        assert_eq!(None, MidiValue::try_from(&[0x83, 2, 64, 3]));
        assert_eq!(None, MidiValue::try_from(&[0x82, 7, 96, 2]));
        assert_eq!(None, MidiValue::try_from(&[0xB2, 7, 96, 2]));
        assert_eq!(None, MidiValue::try_from(&[0xB0, 7, 96, 2]));
    }
}
