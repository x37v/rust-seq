#![feature(specialization)]
//could have a key -> value where the value is an enumerated item of any of the types we support?
//not super flexible..

type UTime = usize;
type SeqFn = Box<SeqCall>;

trait SeqCall {
    fn seq_call(&mut self, &mut Seq) -> Option<UTime>;
}

impl<F: Fn(&mut Seq) -> Option<UTime>> SeqCall for F {
    fn seq_call(&mut self, s: &mut Seq) -> Option<UTime> {
        (*self)(s)
    }
}

trait SeqSend {
    fn send_usize(&mut self, v: usize) -> ();
}

trait SeqCached<T> {
    fn pop() -> Option<Box<T>>;
    fn push(v: Box<T>) -> ();
}

#[derive(Copy, Clone)]
enum Midi {
    Note {
        chan: u8,
        num: u8,
        vel: u8,
        on: bool,
    },
    CC {
        chan: u8,
        num: u8,
        val: u8,
    },
}

impl SeqCall for Midi {
    fn seq_call(&mut self, _s: &mut Seq) -> Option<UTime> {
        match self {
            &mut Midi::Note {
                ref chan,
                ref num,
                ref vel,
                ref mut on,
            } => {
                println!("note {} {} {} {}", on, chan, num, vel);
                if *on {
                    *on = false;
                    Some(20)
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
    fn pop() -> Option<Box<Midi>> {
        Some(Box::new(Midi::Note {
            chan: 0,
            num: 64,
            vel: 127,
            on: true,
        })) //XXX!!!!
    }

    fn push(_v: Box<Midi>) {}
}

impl<T> SeqSend for T {
    default fn send_usize(&mut self, _v: usize) {}
}

impl SeqSend for Seq {
    fn send_usize(&mut self, v: usize) -> () {
        println!("YES {}", v);
    }
}

/*
struct LLNode<T> {
    next: Option<Box<LLNode<T>>>,
    value: T
}

impl<T> LLNode<T> {
    fn new(v: T) -> Self {
        LLNode { next: None, value: v }
    }

    fn append(&mut self, item: Box<LLNode<T>>) {
        self.next = Some(item);
    }
}
*/

// Fn(context) -> option(utime) [if it gets rescheduled or not]
// context allows for scheduling additional things

struct Seq {
    items: Vec<SeqFn>,
    reserve: Vec<SeqFn>,
}

impl Seq {
    fn new() -> Self {
        Seq {
            items: Vec::new(),
            reserve: Vec::new(),
        }
    }

    fn schedule(&mut self, f: SeqFn) {
        self.items.push(f);
    }

    fn reserve(&mut self, f: SeqFn) {
        self.reserve.push(f);
    }

    fn reserve_pop(&mut self) -> Option<SeqFn> {
        self.reserve.pop()
    }

    fn run(&mut self) {
        //XXX loop while we still have items in the current time slice.
        //abort early if it takes too long?
        println!("run!");
        let l: Vec<SeqFn> = self.items.drain(..).collect();
        for mut f in l {
            if let Some(n) = f.seq_call(self) {
                println!("{}", n);
                self.items.push(f);
            }
        }
    }
}

fn main() {
    let mut seq = Seq::new();

    for i in 1..10 {
        seq.reserve(Box::new(move |_s: &mut Seq| Some(i)));
    }

    seq.send_usize(30);
    //XXX MidiCache::push(Box::new(Midi::Note));

    seq.schedule(Box::new(|s: &mut Seq| {
        let v = Box::new(|s: &mut Seq| {
            if let Some(n) = s.reserve_pop() {
                s.schedule(n);
            }
            if let Some(m) = MidiCache::pop() {
                s.schedule(m);
            }
            Some(20)
        });
        s.schedule(v);
        None
    }));

    for _ in 1..10 {
        seq.run();
    }
}
