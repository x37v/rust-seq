#![cfg_attr(not(feature = "std"), no_std)]

pub mod clock;
pub mod context;
pub mod event;
pub mod graph;
pub mod param;
pub mod pqueue;
pub mod sched;
pub mod tick;

#[cfg(feature = "float32")]
pub type Float = f32;
#[cfg(not(feature = "float32"))]
pub type Float = f64;

//rexport
pub use ::num_traits;
pub use ::spin;
