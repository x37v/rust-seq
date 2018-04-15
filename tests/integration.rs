#![feature(specialization)]
#![feature(nll)]

#[macro_use]
extern crate xnor_seq;

use std::sync::Arc;
use xnor_seq::Sched;
use xnor_seq::sequencer;
use std::{thread, time};

#[test]
fn can() {
    let (mut s, mut exec) = sequencer();

    s.spawn_dispose_thread();

    let y = 234.0;
    let x = boxed_fn!(move |_s: &mut Sched| {
        println!("SODA {}", y);
        Some(2)
    });
    s.schedule(0, x);

    s.schedule(
        41,
        boxed_fn!(move |_s: &mut Sched| {
            println!("YES YES YES");
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

    if let Err(e) = child.join() {
        panic!(e);
    }
}
