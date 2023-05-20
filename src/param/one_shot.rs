use super::{ParamGet, ParamSet};
use core::sync::atomic::{AtomicBool, Ordering};

pub struct OneShot {
    inner: AtomicBool,
}

impl OneShot {
    pub const fn new(state: bool) -> Self {
        Self {
            inner: AtomicBool::new(state),
        }
    }
}

#[cfg(not(feature = "no_compare_exchange"))]
impl<U> ParamGet<bool, U> for OneShot {
    fn get(&self, _user_data: &mut U) -> bool {
        self.inner.swap(false, Ordering::SeqCst)
    }
}

///NOTE: expects that there is only one thread doing the get & set
#[cfg(feature = "no_compare_exchange")]
impl ParamGet<bool> for OneShot {
    fn get(&self) -> bool {
        if self.inner.load(Ordering::SeqCst) {
            self.inner.store(false, Ordering::SeqCst);
            true
        } else {
            false
        }
    }
}

impl<U> ParamSet<bool, U> for OneShot {
    fn set(&self, v: bool, _user_data: &mut U) {
        self.inner.store(v, Ordering::SeqCst);
    }
}

impl Default for OneShot {
    fn default() -> Self {
        Self::new(false)
    }
}
