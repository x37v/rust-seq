#![cfg_attr(feature = "no_std", feature(no_std))]
#![cfg_attr(feature = "no_std", feature(alloc))]
#![cfg_attr(feature = "no_std", feature(global_allocator))]

#[macro_use]
extern crate cfg_if;

//this aren't actually usable outside of std
#[macro_use]
pub mod macros;

mod base;
pub mod binding;
pub mod context;
pub mod graph;
pub mod midi;
pub mod ptr;
pub mod time;
pub mod trigger;
pub use crate::base::*;

cfg_if! {
    if #[cfg(feature = "std")] {

        #[macro_use]
        extern crate sched_macros;


        pub mod executor;
        pub mod util;


        //XXX move to its own crate?
        pub mod quneo_display;

    }
}
