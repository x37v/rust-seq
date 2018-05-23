use UTimePoint;
use SchedCall;
use Sched;

pub enum Midi {
    Note {
        chan: u8,
        num: u8,
        vel: u8,
        on: bool,
        dur: UTimePoint,
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
    pub fn note(&mut self, chan: u8, num: u8, vel: u8, dur: UTimePoint) {
        *self = Midi::Note {
            on: true,
            chan,
            num,
            vel,
            dur,
        };
    }
}

use std::thread;

impl SchedCall for Midi {
    fn sched_call(&mut self, _s: &mut Sched) -> Option<UTimePoint> {
        match self {
            &mut Midi::Note {
                ref chan,
                ref num,
                ref vel,
                ref mut on,
                ref dur,
            } => {
                println!(
                    "note {} {} {} {} {:?}",
                    on,
                    chan,
                    num,
                    vel,
                    thread::current().id()
                );
                if *on {
                    *on = false;
                    Some(*dur)
                } else {
                    None
                }
            }
            &mut Midi::CC { chan, num, val } => {
                println!("cc {} {} {} {:?}", chan, num, val, thread::current().id());
                None
            }
        }
    }
}
