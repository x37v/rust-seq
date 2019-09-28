#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MidiValue {
    NoteOn { chan: u8, num: u8, vel: u8 },
    NoteOff { chan: u8, num: u8, vel: u8 },
    ContCtrl { chan: u8, num: u8, val: u8 },
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

impl MidiValue {
    pub fn iter(&self) -> MidiValueIterator<'_> {
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
                    Some(MidiValue::NoteOn {
                        chan,
                        num: bytes[1],
                        vel: bytes[2],
                    })
                } else if status == MidiStatus::NoteOff as u8 {
                    Some(MidiValue::NoteOff {
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

impl From<MidiStatus> for u8 {
    fn from(v: MidiStatus) -> u8 {
        v as u8
    }
}

pub struct MidiValueIterator<'a> {
    value: &'a MidiValue,
    index: u8,
}

impl<'a> Iterator for MidiValueIterator<'a> {
    type Item = u8;

    fn next(&mut self) -> Option<u8> {
        let r = match self.value {
            MidiValue::NoteOn { chan, num, vel } => match self.index {
                0 => Some(status_byte(MidiStatus::NoteOn, *chan)),
                1 => Some(num.mclamp()),
                2 => Some(vel.mclamp()),
                _ => None,
            },
            MidiValue::NoteOff { chan, num, vel } => match self.index {
                0 => Some(status_byte(MidiStatus::NoteOff, *chan)),
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
            MidiValue::NoteOn { .. } | MidiValue::NoteOff { .. } | MidiValue::ContCtrl { .. } => 3,
        }
    }
}
