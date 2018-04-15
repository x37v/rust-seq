#![feature(specialization)]
#![feature(nll)]

extern crate xnor_llist;
#[macro_use]
extern crate xnor_seq;

use std::sync::Arc;
use xnor_llist::Node;
use xnor_seq::Sched;
use xnor_seq::TimedFn;
use xnor_seq::Seq;
use xnor_seq::SeqCached;
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
    let mut sq = Seq::new();

    let y = 234.0;
    let x = boxed_fn!(move |_s: &mut Sched| {
        println!("SODA {}", y);
        Some(2)
    });
    sq.schedule(20, x);
    sq.schedule(
        30,
        boxed_fn!(move |_s: &mut Sched| {
            println!("YES YES YES");
            None
        }),
    );
    sq.run();
    sq.run();
    sq.run();
}
