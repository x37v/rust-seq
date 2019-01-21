#[macro_use]
extern crate cfg_if;

//this aren't actually usable outside of std
#[macro_use]
pub mod macros;

pub mod binding;
pub mod context;
pub mod graph;
pub mod midi;
pub mod ptr;
pub mod time;
pub mod trigger;

cfg_if! {
    if #[cfg(feature = "std")] {

        #[macro_use]
        extern crate sched_macros;


        mod base;
        pub mod executor;
        pub mod util;


        //XXX move to its own crate?
        pub mod quneo_display;

        pub use crate::base::*;
    }
}
