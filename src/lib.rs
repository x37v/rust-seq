#![feature(nll)]
#[cfg_attr(not(feature = "std"), feature(no_std))]
#[macro_use]
extern crate cfg_if;

pub mod time;

cfg_if! {
    if #[cfg(feature = "std")] {
        extern crate failure;
        extern crate spinlock;
        extern crate xnor_llist;
        extern crate rand;

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
    }
}
