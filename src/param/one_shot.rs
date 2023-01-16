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

///NOTE: expects that there is only one thread doing the get & set
#[cfg(feature = "no_compare_exchange")]
impl ParamGet<bool> for OneShot {
    fn get(&self) -> bool {
        if !self.inner.load(Ordering::SeqCst) {
            self.inner.store(true, Ordering::SeqCst);
            true
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
