extern crate alloc;

use super::*;
use crate::Float;
use core::ops::Deref;
use spin::Mutex;

pub trait Clock: Send {
    fn bpm(&self) -> Float;
    fn set_bpm(&mut self, bpm: Float);

    fn period_micros(&self) -> Float;
    fn set_period_micros(&mut self, period_micros: Float);

    fn ppq(&self) -> usize;
    fn set_ppq(&mut self, ppq: usize);
}

#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "with_serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ClockData {
    pub bpm: Float,
    pub period_micros: Float,
    pub ppq: usize,
}

macro_rules! period_micro {
    ($bpm:expr, $ppq:expr) => {
        60.0e6 / ($bpm * $ppq as Float)
    };
}

/// A builder for ClockData that can happen in a static context.
#[macro_export]
macro_rules! make_clock {
    ($bpm:expr, $ppq:expr) => {
        crate::binding::bpm::ClockData {
            bpm: $bpm,
            period_micros: period_micro!($bpm, $ppq),
            ppq: $ppq,
        }
    };
}

pub struct ClockPeriodMicroBinding<T: Deref<Target = Mutex<dyn Clock>> + Sync + Send + Clone>(
    pub T,
);
pub struct ClockBPMBinding<T: Deref<Target = Mutex<dyn Clock>> + Sync + Send + Clone>(pub T);
pub struct ClockPPQBinding<T: Deref<Target = Mutex<dyn Clock>> + Sync + Send + Clone>(pub T);

impl ClockData {
    pub fn period_micro(bpm: Float, ppq: usize) -> Float {
        period_micro!(bpm, ppq)
    }

    pub fn new(bpm: Float, ppq: usize) -> Self {
        Self {
            bpm,
            period_micros: Self::period_micro(bpm, ppq),
            ppq,
        }
    }
}

impl Default for ClockData {
    fn default() -> Self {
        let bpm = 120.0;
        let ppq = 960;
        Self {
            bpm,
            ppq,
            period_micros: period_micro!(bpm, ppq),
        }
    }
}

impl Clock for ClockData {
    fn bpm(&self) -> Float {
        self.bpm
    }

    fn set_bpm(&mut self, bpm: Float) {
        self.bpm = if bpm < 0.0 { 0.001 } else { bpm };
        self.period_micros = Self::period_micro(self.bpm, self.ppq);
    }

    fn period_micros(&self) -> Float {
        self.period_micros
    }

    fn set_period_micros(&mut self, period_micros: Float) {
        self.period_micros = if period_micros < 0.001 {
            0.001
        } else {
            period_micros
        };
        self.bpm = 60.0e6 / (self.period_micros * self.ppq as Float);
    }

    fn ppq(&self) -> usize {
        self.ppq
    }

    fn set_ppq(&mut self, ppq: usize) {
        self.ppq = if ppq < 1 { 1 } else { ppq };
        self.period_micros = Self::period_micro(self.bpm, self.ppq);
    }
}

impl<T> Clone for ClockPeriodMicroBinding<T>
where
    T: Deref<Target = Mutex<dyn Clock>> + Sync + Send + Clone,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> Clone for ClockBPMBinding<T>
where
    T: Deref<Target = Mutex<dyn Clock>> + Sync + Send + Clone,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> Clone for ClockPPQBinding<T>
where
    T: Deref<Target = Mutex<dyn Clock>> + Sync + Send + Clone,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> ParamBindingSet<Float> for ClockPeriodMicroBinding<T>
where
    T: Deref<Target = Mutex<dyn Clock>> + Sync + Send + Clone,
{
    fn set(&self, value: Float) {
        self.0.lock().set_period_micros(value);
    }
}

impl<T> ParamBindingGet<Float> for ClockPeriodMicroBinding<T>
where
    T: Deref<Target = Mutex<dyn Clock>> + Sync + Send + Clone,
{
    fn get(&self) -> Float {
        self.0.lock().period_micros()
    }
}

impl<T> ParamBindingSet<Float> for ClockBPMBinding<T>
where
    T: Deref<Target = Mutex<dyn Clock>> + Sync + Send + Clone,
{
    fn set(&self, value: Float) {
        self.0.lock().set_bpm(value);
    }
}

impl<T> ParamBindingGet<Float> for ClockBPMBinding<T>
where
    T: Deref<Target = Mutex<dyn Clock>> + Sync + Send + Clone,
{
    fn get(&self) -> Float {
        self.0.lock().bpm()
    }
}

impl<T> ParamBindingSet<usize> for ClockPPQBinding<T>
where
    T: Deref<Target = Mutex<dyn Clock>> + Sync + Send + Clone,
{
    fn set(&self, value: usize) {
        self.0.lock().set_ppq(value);
    }
}

impl<T> ParamBindingGet<usize> for ClockPPQBinding<T>
where
    T: Deref<Target = Mutex<dyn Clock>> + Sync + Send + Clone,
{
    fn get(&self) -> usize {
        self.0.lock().ppq()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binding::ParamBindingGet;
    use alloc::sync::Arc;

    use spin::Mutex;

    #[test]
    fn bpm_value_test() {
        assert_eq!(5208f64, bpm::ClockData::period_micro(120.0, 96).floor());
        assert_eq!(20833f64, bpm::ClockData::period_micro(120.0, 24).floor());

        let mut c = bpm::ClockData::new(120.0, 96);
        assert_eq!(5208f64, c.period_micros().floor());
        assert_eq!(120f64, c.bpm());
        assert_eq!(96, c.ppq());

        c.set_ppq(24);
        assert_eq!(20833f64, c.period_micros().floor());
        assert_eq!(120f64, c.bpm());
        assert_eq!(24, c.ppq());

        c.set_bpm(2.0);
        c.set_ppq(96);
        assert_eq!(2f64, c.bpm());
        assert_eq!(96, c.ppq());
        assert_ne!(5208f64, c.period_micros().floor());

        c.set_period_micros(5_208.333333f64);
        assert_eq!(120f64, c.bpm().floor());
        assert_eq!(96, c.ppq());
        assert_eq!(5208f64, c.period_micros().floor());
    }

    #[test]
    fn bpm_binding_test() {
        let b: Arc<Mutex<dyn Clock>> = Arc::new(Mutex::new(bpm::ClockData::new(120.0, 96)));

        let bpm = bpm::ClockBPMBinding(b.clone());
        let ppq = bpm::ClockPPQBinding(b.clone());
        let micros = bpm::ClockPeriodMicroBinding(b.clone());
        let micros2 = micros.clone();

        let c = b.clone();
        assert_eq!(5208f64, c.lock().period_micros().floor());
        assert_eq!(5208f64, micros.get().floor());
        assert_eq!(120f64, c.lock().bpm());
        assert_eq!(120f64, bpm.get());
        assert_eq!(96, c.lock().ppq());
        assert_eq!(96, ppq.get());

        ppq.set(24);
        assert_eq!(20833f64, c.lock().period_micros().floor());
        assert_eq!(20833f64, micros.get().floor());
        assert_eq!(120f64, c.lock().bpm());
        assert_eq!(120f64, bpm.get());
        assert_eq!(24, c.lock().ppq());
        assert_eq!(24, ppq.get());

        bpm.set(2.0);
        ppq.set(96);
        assert_eq!(2f64, c.lock().bpm());
        assert_eq!(2f64, bpm.get());
        assert_eq!(96, c.lock().ppq());
        assert_eq!(96, ppq.get());
        assert_ne!(5208f64, c.lock().period_micros().floor());
        assert_ne!(5208f64, micros.get().floor());

        micros2.set(5_208.333333f64);
        assert_eq!(120f64, c.lock().bpm().floor());
        assert_eq!(120f64, bpm.get().floor());
        assert_eq!(96, c.lock().ppq());
        assert_eq!(96, ppq.get());
        assert_eq!(5208f64, c.lock().period_micros().floor());
        assert_eq!(5208f64, micros.get().floor());
    }

    static CLOCK: Mutex<bpm::ClockData> = Mutex::new(make_clock!(120f64, 96));
    #[test]
    fn bpm_binding_static_test() {
        let b = &CLOCK as &'static Mutex<dyn Clock>;
        let bpm = bpm::ClockBPMBinding(b.clone());
        let ppq = bpm::ClockPPQBinding(b.clone());
        let micros = bpm::ClockPeriodMicroBinding(b.clone());
        let micros2 = micros.clone();

        let c = b.clone();
        assert_eq!(5208f64, c.lock().period_micros().floor());
        assert_eq!(5208f64, micros.get().floor());
        assert_eq!(120f64, c.lock().bpm());
        assert_eq!(120f64, bpm.get());
        assert_eq!(96, c.lock().ppq());
        assert_eq!(96, ppq.get());

        ppq.set(24);
        assert_eq!(20833f64, c.lock().period_micros().floor());
        assert_eq!(20833f64, micros.get().floor());
        assert_eq!(120f64, c.lock().bpm());
        assert_eq!(120f64, bpm.get());
        assert_eq!(24, c.lock().ppq());
        assert_eq!(24, ppq.get());

        bpm.set(2.0);
        ppq.set(96);
        assert_eq!(2f64, c.lock().bpm());
        assert_eq!(2f64, bpm.get());
        assert_eq!(96, c.lock().ppq());
        assert_eq!(96, ppq.get());
        assert_ne!(5208f64, c.lock().period_micros().floor());
        assert_ne!(5208f64, micros.get().floor());

        micros2.set(5_208.333333f64);
        assert_eq!(120f64, c.lock().bpm().floor());
        assert_eq!(120f64, bpm.get().floor());
        assert_eq!(96, c.lock().ppq());
        assert_eq!(96, ppq.get());
        assert_eq!(5208f64, c.lock().period_micros().floor());
        assert_eq!(5208f64, micros.get().floor());
    }
}
