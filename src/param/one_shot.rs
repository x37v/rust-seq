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

impl ParamGet<bool> for OneShot {
    fn get(&self) -> bool {
        self.inner.swap(false, Ordering::SeqCst)
    }
}

impl ParamSet<bool> for OneShot {
    fn set(&self, v: bool) {
        self.inner.store(v, Ordering::SeqCst);
    }
}

impl Default for OneShot {
    fn default() -> Self {
        Self::new(false)
    }
}
