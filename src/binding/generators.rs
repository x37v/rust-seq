use crate::binding::{ParamBindingGet, ParamBindingSet};
use core::marker::PhantomData;
use core::sync::atomic::AtomicBool;

#[cfg(feature = "std")]
use rand::prelude::*;

#[cfg(feature = "euclidean")]
include!(concat!(env!("OUT_DIR"), "/euclid.rs"));

/// Get an random numeric value with the given distribution.
pub struct GetRand<T, R> {
    rng: R,
    _phantom: PhantomData<fn() -> T>,
}

/// Get a One Shot, if set to true, is only true for one read until it is set true again
pub struct GetOneShot {
    binding: AtomicBool,
}

/// Get a Euclidean boolean.
#[cfg(feature = "euclidean")]
pub struct GetEuclid<I, P, S> {
    index: I,
    pulses: P,
    steps: S,
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

#[cfg(feature = "euclidean")]
impl<I, P, S> ParamBindingGet<bool> for GetEuclid<I, P, S>
where
    I: ParamBindingGet<usize>,
    P: ParamBindingGet<usize>,
    S: ParamBindingGet<usize>,
{
    fn get(&self) -> bool {
        //known that we can only do steps up to 64
        let steps = std::cmp::min(64, self.steps.get());
        let pulses = self.pulses.get();
        if steps == 0 || pulses == 0 {
            false
        } else if pulses >= steps {
            true
        } else {
            let index = self.index.get() % steps;
            //get the pattern, it is a bit field
            if let Some(pattern) = EUCLID_STEP_PULSE_PATTERN_MAP.get(&(steps, pulses)) {
                (pattern & (1 << index)) != 0
            } else {
                panic!(
                    "steps: {} pulses: {} should produce a valid pattern",
                    steps, pulses
                );
            }
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
