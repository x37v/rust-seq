use super::*;
use core::cell::Cell;
use spin::Mutex;

/// Wrap any `Copy` type in a `spin::Mutex` so it can be shared across threads.
///
/// *Note*: `Atomic.*` types automatically implement `ParamBindingGet` and `ParamBindingSet`
/// so you probably want to use those when you can.

pub struct SpinlockParamBinding<T: Copy> {
    lock: Mutex<Cell<T>>,
}

impl<T: Copy> SpinlockParamBinding<T> {
    pub fn new(value: T) -> Self {
        SpinlockParamBinding {
            lock: Mutex::new(Cell::new(value)),
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
