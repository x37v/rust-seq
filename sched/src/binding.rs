extern crate spinlock;
extern crate xnor_llist;

pub use xnor_llist::List as LList;
pub use xnor_llist::Node as LNode;

use std::cell::Cell;
use std::sync::Arc;

pub type ValueSetP = Box<dyn ValueSetBinding>;
pub type BindingP<T> = Arc<dyn ParamBinding<T>>;

pub trait ParamBinding<T>: Send + Sync {
    fn set(&self, value: T);
    fn get(&self) -> T;
}

pub trait ValueSetBinding: Send {
    //store the value into the binding
    fn store(&self);
}

pub struct SpinlockParamBinding<T: Copy> {
    lock: spinlock::Mutex<Cell<T>>,
}

pub struct SpinlockValueSetBinding<T: Copy> {
    binding: BindingP<T>,
    value: T,
}

pub struct BPMClockBinding {
    data: BPMClockBindingData,
}

pub struct BPMClockBindingData {
    bpm: f32,
    period_micros: f32,
    ppq: usize,
}

impl<T: Copy> SpinlockParamBinding<T> {
    pub fn new(value: T) -> Self {
        SpinlockParamBinding {
            lock: spinlock::Mutex::new(Cell::new(value)),
        }
    }
}

impl<T: Copy + Send> ParamBinding<T> for SpinlockParamBinding<T> {
    fn set(&self, value: T) {
        self.lock.lock().set(value);
    }

    fn get(&self) -> T {
        self.lock.lock().get()
    }
}

impl<T: Copy> SpinlockValueSetBinding<T> {
    pub fn new(binding: BindingP<T>, value: T) -> Self {
        SpinlockValueSetBinding { binding, value }
    }
}

impl<T: Copy + Send> ValueSetBinding for SpinlockValueSetBinding<T> {
    fn store(&self) {
        self.binding.set(self.value);
    }
}

impl BPMClockBinding {
    pub fn new(bpm: f32, ppq: usize) -> Self {
        Self {
            data: BPMClockBindingData::new(bpm, ppq),
        }
    }

    pub fn bpm_binding(&self) -> Arc<impl ParamBinding<f32>> {
        //XXX TMP/FAKE
        Arc::new(SpinlockParamBinding::new(150f32))
    }

    pub fn ppq_binding(&self) -> Arc<impl ParamBinding<usize>> {
        //XXX TMP/FAKE
        Arc::new(SpinlockParamBinding::new(960usize))
    }

    pub fn period_micro_binding(&self) -> Arc<impl ParamBinding<f32>> {
        //XXX TMP/FAKE
        Arc::new(SpinlockParamBinding::new(15f32))
    }
}

impl BPMClockBindingData {
    fn period_micro(bpm: f32, ppq: usize) -> f32 {
        60e6f32 / (bpm * ppq as f32)
    }
    fn new(bpm: f32, ppq: usize) -> Self {
        Self {
            bpm,
            period_micros: Self::period_micro(bpm, ppq),
            ppq,
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn value_set_binding() {
        let pb = Arc::new(SpinlockParamBinding::new(23));
        assert_eq!(23, pb.get());

        let vsb = SpinlockValueSetBinding::new(pb.clone(), 2084);

        //doesn't change it immediately
        assert_eq!(23, pb.get());

        vsb.store();
        assert_eq!(2084, pb.get());

        pb.set(1);
        assert_eq!(1, pb.get());

        vsb.store();
        assert_eq!(2084, pb.get());
    }

    #[test]
    fn bpm_value_test() {
        assert_eq!(
            5208f32,
            BPMClockBindingData::period_micro(120.0, 96).floor()
        );
        assert_eq!(
            20833.0f32,
            BPMClockBindingData::period_micro(120.0, 24).floor()
        );
    }

}
