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
        if let Ok(state) =
            self.inner
                .compare_exchange(true, false, Ordering::SeqCst, Ordering::SeqCst)
        {
            state
        } else {
            false
        }
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
