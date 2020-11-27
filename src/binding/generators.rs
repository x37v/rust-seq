use crate::binding::{ParamBindingGet, ParamBindingSet};
use core::marker::PhantomData;
use core::sync::atomic::AtomicBool;

#[cfg(feature = "std")]
use rand::prelude::*;

/// Get an uniform random numeric value [min, max(.
///
/// This generates a new random value that is greater than or equal to `min` and less than `max`
/// every time you call `.get()` on it.
pub struct GetUniformRand<T, Min, Max> {
    min: Min,
    max: Max,
    _phantom: PhantomData<fn() -> T>,
}

/// Get a One Shot, if set to true, is only true for one read until it is set true again
pub struct GetOneShot {
    binding: AtomicBool,
}

impl<T, Min, Max> GetUniformRand<T, Min, Max>
where
    T: Send,
    Min: ParamBindingGet<T>,
    Max: ParamBindingGet<T>,
{
    /// Construct a new `GetUniformRand`
    ///
    /// # Arguments
    ///
    /// * `min` - the binding for the minimum value
    /// * `max` - the binding for the maximum value
    ///
    /// # Notes
    /// The max is **exclusive** so you will never get that value in the output.
    pub fn new(min: Min, max: Max) -> Self {
        Self {
            min,
            max,
            _phantom: Default::default(),
        }
    }
}

#[cfg(feature = "std")]
impl<T, Min, Max> ParamBindingGet<T> for GetUniformRand<T, Min, Max>
where
    T: rand::distributions::uniform::SampleUniform + PartialOrd,
    Min: ParamBindingGet<T>,
    Max: ParamBindingGet<T>,
{
    fn get(&self) -> T {
        let min = self.min.get();
        let max = self.max.get();
        if min >= max {
            min
        } else {
            thread_rng().gen_range(min, max)
        }
    }
}

impl GetOneShot {
    /// Construct a new `GetOneShot`
    pub fn new() -> Self {
        Self {
            binding: AtomicBool::new(false),
        }
    }
}

impl Default for GetOneShot {
    fn default() -> Self {
        Self::new()
    }
}

impl ParamBindingGet<bool> for GetOneShot {
    fn get(&self) -> bool {
        if self.binding.get() {
            self.binding.set(false);
            true
        } else {
            false
        }
    }
}

impl ParamBindingSet<bool> for GetOneShot {
    fn set(&self, value: bool) {
        self.binding.set(value);
    }
}
