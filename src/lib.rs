#![feature(nll)]

extern crate failure;
extern crate rand;
pub extern crate spinlock;
pub extern crate xnor_llist;

#[macro_use]
extern crate sched_macros;

pub mod macros;

mod base;
pub mod context;
pub mod executor;
pub mod graph;
pub mod ptr;
pub mod trigger;
pub mod util;

pub mod binding;
pub mod midi;

//XXX move to its own crate?
pub mod quneo_display;

pub use crate::base::*;
