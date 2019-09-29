#![cfg_attr(not(feature = "std"), no_std)]

pub mod event;
pub mod item_sink;
pub mod item_source;
pub mod midi;
pub mod pqueue;
pub mod schedule;
pub mod time;

cfg_if::cfg_if! {
    if #[cfg(feature = "std")] {
        pub mod std;
    } else {
        pub mod no_std;
    }
}
