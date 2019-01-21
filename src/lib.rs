#[macro_use]
extern crate cfg_if;

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

        pub mod macros;

        mod base;
        pub mod executor;
        pub mod util;


        //XXX move to its own crate?
        pub mod quneo_display;

        pub use crate::base::*;
    }
}
