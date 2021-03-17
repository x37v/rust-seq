#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

pub mod binding;
pub mod context;
pub mod event;
pub mod graph;
pub mod item_sink;
pub mod item_source;
pub mod midi;
pub mod pqueue;
pub mod schedule;
pub mod tick;

//TODO provide an option for f32
pub type Float = f64;

#[cfg(feature = "std")]
pub mod std;

pub use ::atomic;
/// Re-exports
pub use ::spin as mutex;
