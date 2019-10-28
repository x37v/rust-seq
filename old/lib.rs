#![cfg_attr(not(feature = "std"), no_std)]
//#![feature(unsize)]
//#![feature(coerce_unsized)]
//#![feature(dispatch_from_dyn)]

#[macro_use]
extern crate cfg_if;

//these aren't actually usable outside of std
#[macro_use]
pub mod macros;

mod base;
pub mod binding;
pub mod context;
pub mod graph;
pub mod list;
pub mod midi;
pub mod pqueue;
pub mod ptr;
pub mod time;
pub mod trigger;
pub mod util;
pub use crate::base::*;

#[macro_use]
extern crate sched_macros;

cfg_if! {
    if #[cfg(feature = "std")] {
        pub mod executor;
        pub mod llist_pqueue;

        //XXX move to its own crate?
        pub mod quneo_display;

    }
}