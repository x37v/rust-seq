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

pub struct BPMClock {
    bpm: f32,
    period_micros: f32,
    ppq: usize,
}

pub struct BPMClockPeriodMicroBinding(pub Arc<spinlock::Mutex<BPMClock>>);
pub struct BPMClockBPMBinding(pub Arc<spinlock::Mutex<BPMClock>>);
pub struct BPMClockPPQBinding(pub Arc<spinlock::Mutex<BPMClock>>);

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

impl BPMClock {
    pub fn period_micro(bpm: f32, ppq: usize) -> f32 {
        60e6f32 / (bpm * ppq as f32)
    }

    pub fn new(bpm: f32, ppq: usize) -> Self {
        Self {
            bpm,
            period_micros: Self::period_micro(bpm, ppq),
            ppq,
        }
    }

    pub fn bpm(&self) -> f32 {
        self.bpm
    }

    pub fn set_bpm(&mut self, bpm: f32) {
        self.bpm = if bpm < 0f32 { 0.001f32 } else { bpm };
        self.period_micros = Self::period_micro(self.bpm, self.ppq);
    }

    pub fn period_micros(&self) -> f32 {
        self.period_micros
    }

    pub fn set_period_micros(&mut self, period_micros: f32) {
        self.period_micros = if period_micros < 0.001f32 {
            0.001f32
        } else {
            period_micros
        };
        self.bpm = 60e6f32 / (self.period_micros * self.ppq as f32);
    }

    pub fn ppq(&self) -> usize {
        self.ppq
    }

    pub fn set_ppq(&mut self, ppq: usize) {
        self.ppq = if ppq < 1 { 1 } else { ppq };
        self.period_micros = Self::period_micro(self.bpm, self.ppq);
    }
}

impl BPMClockPeriodMicroBinding {
    pub fn new(bpm_clock_binding: Arc<spinlock::Mutex<BPMClock>>) -> Self {
        Self {
            0: bpm_clock_binding,
        }
    }
}

impl BPMClockBPMBinding {
    pub fn new(bpm_clock_binding: Arc<spinlock::Mutex<BPMClock>>) -> Self {
        Self {
            0: bpm_clock_binding,
        }
    }
}

impl BPMClockPPQBinding {
    pub fn new(bpm_clock_binding: Arc<spinlock::Mutex<BPMClock>>) -> Self {
        Self {
            0: bpm_clock_binding,
        }
    }
}

impl ParamBinding<f32> for BPMClockPeriodMicroBinding {
    fn set(&self, value: f32) {
        self.0.lock().set_period_micros(value);
    }
    fn get(&self) -> f32 {
        self.0.lock().period_micros()
    }
}

impl ParamBinding<f32> for BPMClockBPMBinding {
    fn set(&self, value: f32) {
        self.0.lock().set_bpm(value);
    }
    fn get(&self) -> f32 {
        self.0.lock().bpm()
    }
}

impl ParamBinding<usize> for BPMClockPPQBinding {
    fn set(&self, value: usize) {
        self.0.lock().set_ppq(value);
    }
    fn get(&self) -> usize {
        self.0.lock().ppq()
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
        assert_eq!(5208f32, BPMClock::period_micro(120.0, 96).floor());
        assert_eq!(20833f32, BPMClock::period_micro(120.0, 24).floor());

        let mut c = BPMClock::new(120.0, 96);
        assert_eq!(5208f32, c.period_micros().floor());
        assert_eq!(120f32, c.bpm());
        assert_eq!(96, c.ppq());

        c.set_ppq(24);
        assert_eq!(20833f32, c.period_micros().floor());
        assert_eq!(120f32, c.bpm());
        assert_eq!(24, c.ppq());

        c.set_bpm(2.0);
        c.set_ppq(96);
        assert_eq!(2f32, c.bpm());
        assert_eq!(96, c.ppq());
        assert_ne!(5208f32, c.period_micros().floor());

        c.set_period_micros(5_208.333333f32);
        assert_eq!(120f32, c.bpm().floor());
        assert_eq!(96, c.ppq());
        assert_eq!(5208f32, c.period_micros().floor());
    }

    #[test]
    fn bpm_binding_test() {
        let mut b = Arc::new(spinlock::Mutex::new(BPMClock::new(120.0, 96)));

        let c = b.clone();
        assert_eq!(5208f32, c.lock().period_micros().floor());
        assert_eq!(120f32, c.lock().bpm());
        assert_eq!(96, c.lock().ppq());

        c.lock().set_ppq(24);
        assert_eq!(20833f32, c.lock().period_micros().floor());
        assert_eq!(120f32, c.lock().bpm());
        assert_eq!(24, c.lock().ppq());

        c.lock().set_bpm(2.0);
        c.lock().set_ppq(96);
        assert_eq!(2f32, c.lock().bpm());
        assert_eq!(96, c.lock().ppq());
        assert_ne!(5208f32, c.lock().period_micros().floor());

        c.lock().set_period_micros(5_208.333333f32);
        assert_eq!(120f32, c.lock().bpm().floor());
        assert_eq!(96, c.lock().ppq());
        assert_eq!(5208f32, c.lock().period_micros().floor());
    }

}
