#![feature(specialization)]
#![feature(nll)]
//could have a key -> value where the value is an enumerated item of any of the types we support?
//not super flexible..

use std::sync::Arc;

type TimePoint = isize;
type SeqFn = Arc<SeqCall>;

trait SeqCall {
    fn seq_call(&mut self, &mut Seq) -> Option<TimePoint>;
}

impl<F: Fn(&mut Seq) -> Option<TimePoint>> SeqCall for F {
    fn seq_call(&mut self, s: &mut Seq) -> Option<TimePoint> {
        (*self)(s)
    }
}

trait SeqCached<T> {
    fn pop() -> Option<Arc<T>>;
    fn push(v: Arc<T>) -> ();
}

#[derive(Copy, Clone)]
enum Midi {
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
    fn note(&mut self, chan: u8, num: u8, vel: u8, dur: TimePoint) {
        *self = Midi::Note {
            on: true,
            chan,
            num,
            vel,
            dur,
        };
    }
}

impl SeqCall for Midi {
    fn seq_call(&mut self, _s: &mut Seq) -> Option<TimePoint> {
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

struct MidiCache;

impl SeqCached<Midi> for MidiCache {
    fn pop() -> Option<Arc<Midi>> {
        Some(Arc::new(Midi::default()))
    }

    fn push(_v: Arc<Midi>) {}
}

/*
trait SeqSend {
    fn send_usize(&mut self, v: usize) -> ();
}

impl<T> SeqSend for T {
    default fn send_usize(&mut self, _v: usize) {}
}

impl SeqSend for Seq {
    fn send_usize(&mut self, v: usize) -> () {
        println!("YES {}", v);
    }
}
*/

/*
struct LLNode<T> {
    next: Option<Arc<LLNode<T>>>,
    value: T
}

impl<T> LLNode<T> {
    fn new(v: T) -> Self {
        LLNode { next: None, value: v }
    }

    fn append(&mut self, item: Arc<LLNode<T>>) {
        self.next = Some(item);
    }
}
*/

// Fn(context) -> option(utime) [if it gets rescheduled or not]
// context allows for scheduling additional things

struct Seq {
    items: Vec<SeqFn>,
}

impl Seq {
    fn new() -> Self {
        Seq { items: Vec::new() }
    }

    fn schedule(&mut self, _t: TimePoint, f: SeqFn) {
        self.items.push(f);
    }

    fn run(&mut self) {
        println!("run!");
        let l: Vec<SeqFn> = self.items.drain(..).collect();
        for mut f in l {
            if let Some(fm) = Arc::get_mut(&mut f) {
                if let Some(_n) = fm.seq_call(self) {
                    self.items.push(f);
                }
            }
        }
    }
}

fn main() {
    let mut seq = Seq::new();

    //XXX MidiCache::push(Arc::new(Midi::Note));

    seq.schedule(
        0,
        Arc::new(|s: &mut Seq| {
            let v = Arc::new(|s: &mut Seq| {
                if let Some(mut m) = MidiCache::pop() {
                    if let Some(mm) = Arc::get_mut(&mut m) {
                        mm.note(0, 1, 127, 64);
                        s.schedule(0, m);
                    }
                }
                Some(20)
            });
            s.schedule(0, v);
            None
        }),
    );

    for _ in 1..10 {
        seq.run();
    }
}
