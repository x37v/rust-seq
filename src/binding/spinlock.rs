extern crate spinlock;
use super::*;
use ptr::ShrPtr;
use std::cell::Cell;

pub type SpinlockParamBindingP<T> = ShrPtr<SpinlockParamBinding<T>>;

/// Wrap any `Copy` type in a `spinlock::Mutex` so it can be shared across threads.
///
/// *Note*: `AtomicBool`, `AtomicUsize`, and `AtomicIsize` `ParamBindingGet` and `ParamBindingSet`
/// implementations exist, these are be better to use for `bool`, `usize` and `isize` wrapping.

pub struct SpinlockParamBinding<T: Copy> {
    lock: spinlock::Mutex<Cell<T>>,
}

impl<T: Copy> SpinlockParamBinding<T> {
    pub fn new(value: T) -> Self {
        SpinlockParamBinding {
            lock: spinlock::Mutex::new(Cell::new(value)),
        }
    }
}

impl<T: Copy + Send> ParamBindingSet<T> for SpinlockParamBinding<T> {
    fn set(&self, value: T) {
        self.lock.lock().set(value);
    }
}

impl<T: Copy + Send> ParamBindingGet<T> for SpinlockParamBinding<T> {
    fn get(&self) -> T {
        self.lock.lock().get()
    }
}

impl<T> Default for SpinlockParamBinding<T>
where
    T: Default + Copy,
{
    fn default() -> Self {
        Self::new(Default::default())
    }
}
