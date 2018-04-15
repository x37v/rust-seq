#![feature(specialization)]
#![feature(nll)]

#[macro_use]
extern crate xnor_seq;

use std::sync::Arc;
use xnor_seq::Sched;
use xnor_seq::sequencer;
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

#[test]
fn can() {
    let (mut s, mut exec) = sequencer();

    s.spawn_dispose_thread();

    let y = 234.0;
    let x = boxed_fn!(move |_s: &mut Sched| {
        println!("SODA {}", y);
        Some(2)
    });
    s.schedule(20, x);
    s.schedule(
        30,
        boxed_fn!(move |_s: &mut Sched| {
            println!("YES YES YES");
            None
        }),
    );

    let child = thread::spawn(move || {
        exec.run();
        exec.run();
        exec.run();
    });

    if let Err(e) = child.join() {
        panic!(e);
    }
}