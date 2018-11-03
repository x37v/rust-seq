extern crate spinlock;
extern crate xnor_llist;

pub use xnor_llist::List as LList;
pub use xnor_llist::Node as LNode;

use std::cell::Cell;
use std::sync::Arc;

pub type BindingP<T> = Arc<dyn ParamBinding<T>>;
pub type BindingGetP<T> = Arc<dyn ParamBindingGet<T>>;
pub type BindingSetP<T> = Arc<dyn ParamBindingSet<T>>;
pub type SpinlockParamBindingP<T> = Arc<SpinlockParamBinding<T>>;

pub trait ParamBindingGet<T>: Send + Sync {
    fn get(&self) -> T;
}

pub trait ParamBindingSet<T>: Send + Sync {
    fn set(&self, value: T);
}

pub trait ParamBindingLatch {
    fn store(&self);
}

pub trait ParamBinding<T>: ParamBindingSet<T> + ParamBindingGet<T> {}

//a binding and a value to set it to
#[derive(Clone)]
pub enum ValueSet {
    None,
    F32(f32, BindingSetP<f32>),
    I32(i32, BindingSetP<i32>),
    U8(u8, BindingSetP<u8>),
    BOOL(bool, BindingSetP<bool>),
}

impl ValueSet {
    pub fn store(&self) {
        match self {
            ValueSet::None => (),
            ValueSet::F32(v, b) => b.set(*v),
            ValueSet::I32(v, b) => b.set(*v),
            ValueSet::U8(v, b) => b.set(*v),
            ValueSet::BOOL(v, b) => b.set(*v),
        }
    }
}

pub struct ValueLatch<T> {
    get: BindingGetP<T>,
    set: BindingSetP<T>,
}

pub struct AggregateValueLatch {
    latches: Vec<Arc<dyn ParamBindingLatch>>,
}

impl<T> ValueLatch<T> {
    pub fn new(get: BindingGetP<T>, set: BindingSetP<T>) -> Self {
        Self { get, set }
    }
}

impl AggregateValueLatch {
    pub fn new(latches: Vec<Arc<dyn ParamBindingLatch>>) -> Self {
        Self { latches }
    }
}

impl<T> ParamBindingLatch for ValueLatch<T> {
    fn store(&self) {
        self.set.set(self.get.get());
    }
}

impl ParamBindingLatch for AggregateValueLatch {
    fn store(&self) {
        for l in self.latches.iter() {
            l.store();
        }
    }
}

pub struct SpinlockParamBinding<T: Copy> {
    lock: spinlock::Mutex<Cell<T>>,
}

impl Default for ValueSet {
    fn default() -> Self {
        ValueSet::None
    }
}

impl<T: Copy> SpinlockParamBinding<T> {
    pub fn new(value: T) -> Self {
        SpinlockParamBinding {
            lock: spinlock::Mutex::new(Cell::new(value)),
        }
    }
    pub fn new_p(value: T) -> Arc<Self> {
        Arc::new(Self::new(value))
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

pub mod bpm {
    use super::*;
    extern crate spinlock;
    use std::sync::Arc;

    pub struct ClockData {
        bpm: f32,
        period_micros: f32,
        ppq: usize,
    }

    pub struct ClockPeriodMicroBinding(pub Arc<spinlock::Mutex<ClockData>>);
    pub struct ClockBPMBinding(pub Arc<spinlock::Mutex<ClockData>>);
    pub struct ClockPPQBinding(pub Arc<spinlock::Mutex<ClockData>>);

    impl ClockData {
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

    impl ParamBindingSet<f32> for ClockPeriodMicroBinding {
        fn set(&self, value: f32) {
            self.0.lock().set_period_micros(value);
        }
    }

    impl ParamBindingGet<f32> for ClockPeriodMicroBinding {
        fn get(&self) -> f32 {
            self.0.lock().period_micros()
        }
    }

    impl ParamBindingSet<f32> for ClockBPMBinding {
        fn set(&self, value: f32) {
            self.0.lock().set_bpm(value);
        }
    }

    impl ParamBindingGet<f32> for ClockBPMBinding {
        fn get(&self) -> f32 {
            self.0.lock().bpm()
        }
    }

    impl ParamBindingSet<usize> for ClockPPQBinding {
        fn set(&self, value: usize) {
            self.0.lock().set_ppq(value);
        }
    }

    impl ParamBindingGet<usize> for ClockPPQBinding {
        fn get(&self) -> usize {
            self.0.lock().ppq()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bpm_value_test() {
        assert_eq!(5208f32, bpm::ClockData::period_micro(120.0, 96).floor());
        assert_eq!(20833f32, bpm::ClockData::period_micro(120.0, 24).floor());

        let mut c = bpm::ClockData::new(120.0, 96);
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
        let b = Arc::new(spinlock::Mutex::new(bpm::ClockData::new(120.0, 96)));

        let bpm = Arc::new(bpm::ClockBPMBinding(b.clone()));
        let ppq = Arc::new(bpm::ClockPPQBinding(b.clone()));
        let micros = Arc::new(bpm::ClockPeriodMicroBinding(b.clone()));
        let micros2 = micros.clone();

        let c = b.clone();
        assert_eq!(5208f32, c.lock().period_micros().floor());
        assert_eq!(5208f32, micros.get().floor());
        assert_eq!(120f32, c.lock().bpm());
        assert_eq!(120f32, bpm.get());
        assert_eq!(96, c.lock().ppq());
        assert_eq!(96, ppq.get());

        ppq.set(24);
        assert_eq!(20833f32, c.lock().period_micros().floor());
        assert_eq!(20833f32, micros.get().floor());
        assert_eq!(120f32, c.lock().bpm());
        assert_eq!(120f32, bpm.get());
        assert_eq!(24, c.lock().ppq());
        assert_eq!(24, ppq.get());

        bpm.set(2.0);
        ppq.set(96);
        assert_eq!(2f32, c.lock().bpm());
        assert_eq!(2f32, bpm.get());
        assert_eq!(96, c.lock().ppq());
        assert_eq!(96, ppq.get());
        assert_ne!(5208f32, c.lock().period_micros().floor());
        assert_ne!(5208f32, micros.get().floor());

        micros2.set(5_208.333333f32);
        assert_eq!(120f32, c.lock().bpm().floor());
        assert_eq!(120f32, bpm.get().floor());
        assert_eq!(96, c.lock().ppq());
        assert_eq!(96, ppq.get());
        assert_eq!(5208f32, c.lock().period_micros().floor());
        assert_eq!(5208f32, micros.get().floor());
    }

}