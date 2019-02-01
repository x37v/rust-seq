use super::*;
use core::sync::atomic::{AtomicBool, AtomicIsize, AtomicUsize, Ordering};

/// Implementations of `ParamBindingGet` and `ParamBindingSet` for `AtomicBool`, `AtomicUsize`,
/// and `AtomicIsize`

const GET_ORDERING: Ordering = Ordering::SeqCst;
const SET_ORDERING: Ordering = Ordering::SeqCst;

impl ParamBindingGet<usize> for AtomicUsize {
    fn get(&self) -> usize {
        self.load(GET_ORDERING)
    }
}

impl ParamBindingGet<isize> for AtomicIsize {
    fn get(&self) -> isize {
        self.load(GET_ORDERING)
    }
}

impl ParamBindingGet<bool> for AtomicBool {
    fn get(&self) -> bool {
        self.load(GET_ORDERING)
    }
}

impl ParamBindingSet<usize> for AtomicUsize {
    fn set(&self, value: usize) {
        self.store(value, SET_ORDERING);
    }
}
impl ParamBindingSet<isize> for AtomicIsize {
    fn set(&self, value: isize) {
        self.store(value, SET_ORDERING);
    }
}

impl ParamBindingSet<bool> for AtomicBool {
    fn set(&self, value: bool) {
        self.store(value, SET_ORDERING);
    }
}
