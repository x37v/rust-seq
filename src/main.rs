#![feature(specialization)]
#![feature(nll)]

pub mod seq;
pub mod midi;
pub mod llist;

use std::sync::Arc;
use seq::Sched;
use seq::Seq;
use seq::SeqCached;
use std::thread;

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

//XXX instead of disposing back to the cache there can be a thread that keeps each cache full!

fn main() {
    let mut sq = Seq::new();

    //XXX midi::MidiCache::push(Arc::new(Midi::Note));
    
    let y = 234.0;
    let x = Arc::new(move |_s: &mut seq::Sched| {
        println!("SODA {} {}", y);
        None
    });
    sq.schedule(20, x);

    sq.schedule(
        0,
        Arc::new(|s: &mut seq::Sched| {
            let v = Arc::new(|s: &mut seq::Sched| {
                if let Some(mut m) = midi::MidiCache::pop() {
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


    let child = thread::spawn(move || {
        for _ in 1..10 {
            sq.run();
        }
    });
    child.join();
}
