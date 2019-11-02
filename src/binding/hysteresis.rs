use crate::binding::ParamBindingGet;
use core::cell::Cell;
use num::traits::float::FloatCore;
use spin::Mutex;

pub struct Hysteresis<T, B> {
    binding: B,
    threshold: T,
    last: Mutex<Cell<T>>,
}

impl<T, B> Hysteresis<T, B>
where
    T: FloatCore + Send + Sync,
    B: ParamBindingGet<T>,
{
    pub fn new(binding: B, threshold: T) -> Self {
        let last = Mutex::new(Cell::new(binding.get()));
        Self {
            binding: binding,
            threshold,
            last,
        }
    }
}

impl<T, B> ParamBindingGet<T> for Hysteresis<T, B>
where
    T: FloatCore + Send + Sync,
    B: ParamBindingGet<T>,
{
    fn get(&self) -> T {
        let c = self.binding.get();
        let l = self.last.lock();
        l.get()
    }
}

#[cfg(test)]
mod tests {
    extern crate alloc;
    use super::*;
    use crate::binding::{ParamBindingGet, ParamBindingSet};
    use alloc::sync::Arc;
    use assert_approx_eq::assert_approx_eq;

    #[test]
    fn hysteresis() {
        let t = 0.1f32;
        let b = Arc::new(Mutex::new(
            crate::binding::spinlock::SpinlockParamBinding::new(0f32),
        ));
        let h = Hysteresis::new(b.clone() as Arc<Mutex<dyn ParamBindingGet<f32>>>, t);
        assert_approx_eq!(0f32, h.get());

        let b = b as Arc<Mutex<dyn ParamBindingSet<f32>>>;

        b.set(1.09f32);
        assert_approx_eq!(0f32, h.get());

        b.set(-0.09f32);
        assert_approx_eq!(0f32, h.get());

        b.set(1.11f32);
        assert_approx_eq!(1f32, h.get());

        b.set(1.1f32);
        assert_approx_eq!(1f32, h.get());

        b.set(0.99f32);
        assert_approx_eq!(1f32, h.get());

        b.set(0.9f32);
        assert_approx_eq!(0f32, h.get());

        b.set(0.95f32);
        assert_approx_eq!(0f32, h.get());

        b.set(1.0f32);
        assert_approx_eq!(0f32, h.get());

        b.set(1.1f32);
        assert_approx_eq!(1f32, h.get());

        b.set(2.1f32);
        assert_approx_eq!(2f32, h.get());

        b.set(2.1f32);
        assert_approx_eq!(2f32, h.get());

        b.set(2.01f32);
        assert_approx_eq!(2f32, h.get());

        b.set(3.09f32);
        assert_approx_eq!(2f32, h.get());

        b.set(-3.0f32);
        assert_approx_eq!(-2f32, h.get());

        b.set(-3.1f32);
        assert_approx_eq!(-3f32, h.get());
    }
}
