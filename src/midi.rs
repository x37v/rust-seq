use std::sync::Arc;

use seq::TimePoint;
use seq::SchedCall;
use seq::Sched;
use seq::SeqCached;

#[derive(Copy, Clone)]
pub enum Midi {
    Note {
        chan: u8,
        num: u8,
        vel: u8,
        on: bool,
        dur: TimePoint,
    },
    CC {
        chan: u8,
        num: u8,
        val: u8,
    },
}

impl Default for Midi {
    fn default() -> Midi {
        Midi::Note {
            chan: 0,
            num: 64,
            vel: 127,
            on: false,
            dur: 0,
        }
    }
}

impl Midi {
    pub fn note(&mut self, chan: u8, num: u8, vel: u8, dur: TimePoint) {
        *self = Midi::Note {
            on: true,
            chan,
            num,
            vel,
            dur,
        };
    }
}

impl SchedCall for Midi {
    fn sched_call(&mut self, _s: &mut Sched) -> Option<TimePoint> {
        match self {
            &mut Midi::Note {
                ref chan,
                ref num,
                ref vel,
                ref mut on,
                ref dur,
            } => {
                println!("note {} {} {} {}", on, chan, num, vel);
                if *on {
                    *on = false;
                    Some(*dur)
                } else {
                    None
                }
            }
            &mut Midi::CC { chan, num, val } => {
                println!("cc {} {} {}", chan, num, val);
                None
            }
        }
    }
}

pub struct MidiCache;

impl SeqCached<Midi> for MidiCache {
    fn pop() -> Option<Arc<Midi>> {
        Some(Arc::new(Midi::default()))
    }

    fn push(_v: Arc<Midi>) {}
}
