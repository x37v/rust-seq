#![feature(specialization)]
#![feature(nll)]

#[macro_use]
extern crate xnor_seq;

use std::sync::Arc;
use xnor_seq::Sched;
use xnor_seq::sequencer;
use std::{thread, time};
use std::sync::atomic::{AtomicUsize, Ordering};

#[test]
fn can() {
    let (mut s, mut exec) = sequencer();
    s.spawn_helper_threads();

    //can use atomics!
    let y = Arc::new(AtomicUsize::new(2));
    let c = y.clone();
    let x = wrap_fn!(move |_s: &mut Sched| {
        println!("SODA {}", c.load(Ordering::Relaxed));
        Some(2)
    });
    s.schedule(0, x);

    s.schedule(
        41,
        wrap_fn!(move |s: &mut Sched| {
            println!("YES YES YES");
            s.schedule(
                3,
                //XXX THIS IS A BAD MOVE, WE DON'T WANT TO ALLOCATE IN REMOTE THREAD
                wrap_fn!(move |_s: &mut Sched| {
                    println!("INNER DOG");
                    None
                }),
            );
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
