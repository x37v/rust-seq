#![feature(nll)]

pub extern crate spinlock;
pub extern crate xnor_llist;

mod base;
pub mod graph;

pub use base::*;
