use crate::binding::{ParamBindingGet, ParamBindingSet};
use core::marker::PhantomData;
use core::sync::atomic::AtomicBool;

#[cfg(feature = "std")]
use rand::prelude::*;

/// Get an random numeric value with the given distribution.
pub struct GetRand<T, R> {
    rng: R,
    _phantom: PhantomData<fn() -> T>,
}

/// Get a One Shot, if set to true, is only true for one read until it is set true again
pub struct GetOneShot {
    binding: AtomicBool,
}

impl<T, R> GetRand<T, R>
where
    T: Send,
    R: rand::distributions::Distribution<T> + Send,
{
    /// Construct a new `GetRand`
    ///
    /// # Arguments
    ///
    /// * `rng` - implementor of rand::distributions::Distribution<T>
    pub fn new(rng: R) -> Self {
        Self {
            rng,
            _phantom: Default::default(),
        }
    }
}

#[cfg(feature = "std")]
impl<T, R> ParamBindingGet<T> for GetRand<T, R>
where
    T: Send,
    R: rand::distributions::Distribution<T> + Send + Sync,
{
    fn get(&self) -> T {
        self.rng.sample(&mut thread_rng())
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn rand() {
        //mostly just making sure we can build random
        let r = Arc::new(GetRand::new(rand::distributions::Uniform::new(1f32, 10f32)));

        let b = r as Arc<dyn ParamBindingGet<f32>>;
        assert!(b.get() >= 1f32);
        assert!(b.get() <= 10f32);
    }
}
