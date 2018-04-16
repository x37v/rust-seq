#![feature(specialization)]
#![feature(nll)]

#[macro_use]
extern crate xnor_seq;

use std::sync::Arc;
use xnor_seq::Sched;
use xnor_seq::sequencer;
use xnor_seq::midi::Midi;
use std::{thread, time};
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct MidiCache;

//XXX can a cache be a channel? Maybe we use a spinlock owned by the sequence and each thread gets
//its own copy?
impl MidiCache {
    fn pop(_s: &mut Sched) -> Option<Box<Midi>> {
        //XXX grab from channel
        Some(Box::new(Midi::default()))
    }

    /*
     * XXX push into channel
    fn push(_v: Box<Midi>) {}
    */
}

#[test]
fn can() {
    let (mut s, mut exec) = sequencer();
    s.spawn_helper_threads();

    //can use atomics!
    let y = Arc::new(AtomicUsize::new(2));
    let c = y.clone();
    let x = wrap_fn!(move |_s: &mut Sched| {
        println!(
            "SODA {} {:?}",
            c.load(Ordering::Relaxed),
            thread::current().id()
        );
        Some(2)
    });
    s.schedule(0, x);

    s.schedule(
        41,
        wrap_fn!(move |s: &mut Sched| {
            println!("YES YES YES, {:?}", thread::current().id());
            if let Some(mut m) = MidiCache::pop(s) {
                m.note(3, 4, 5, 93);
                s.schedule(3, m);
            }
            None
        }),
    );

    let child = thread::spawn(move || {
        let delay = time::Duration::from_millis(20);
        while exec.time() < 200 {
            exec.run(20);
            thread::sleep(delay);
        }
        println!("ditching exec thread");
    });

    thread::sleep(time::Duration::from_millis(40));
    y.store(2084, Ordering::Relaxed);

    if let Err(e) = child.join() {
        panic!(e);
    }
}
