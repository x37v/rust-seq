#![feature(specialization)]
#![feature(nll)]

pub mod seq;
pub mod midi;

use std::sync::Arc;
use seq::Sched;
use seq::Seq;
use seq::SeqCached;

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

fn main() {
    let mut sq = Seq::new();

    //XXX midi::MidiCache::push(Arc::new(Midi::Note));

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

    for _ in 1..10 {
        sq.run();
    }
}
