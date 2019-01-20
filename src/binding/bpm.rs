use super::*;
extern crate spinlock;
use crate::ptr::SShrPtr;

pub trait Clock {
    fn bpm(&self) -> f32;
    fn set_bpm(&mut self, bpm: f32);

    fn period_micros(&self) -> f32;
    fn set_period_micros(&mut self, period_micros: f32);

    fn ppq(&self) -> usize;
    fn set_ppq(&mut self, ppq: usize);
}

#[derive(Debug, Copy, Clone)]
pub struct ClockData {
    bpm: f32,
    period_micros: f32,
    ppq: usize,
}

pub struct ClockPeriodMicroBinding(pub SShrPtr<Clock>);
pub struct ClockBPMBinding(pub SShrPtr<Clock>);
pub struct ClockPPQBinding(pub SShrPtr<Clock>);

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
}

impl Clock for ClockData {
    fn bpm(&self) -> f32 {
        self.bpm
    }

    fn set_bpm(&mut self, bpm: f32) {
        self.bpm = if bpm < 0f32 { 0.001f32 } else { bpm };
        self.period_micros = Self::period_micro(self.bpm, self.ppq);
    }

    fn period_micros(&self) -> f32 {
        self.period_micros
    }

    fn set_period_micros(&mut self, period_micros: f32) {
        self.period_micros = if period_micros < 0.001f32 {
            0.001f32
        } else {
            period_micros
        };
        self.bpm = 60e6f32 / (self.period_micros * self.ppq as f32);
    }

    fn ppq(&self) -> usize {
        self.ppq
    }

    fn set_ppq(&mut self, ppq: usize) {
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
        let b = new_sshrptr!(bpm::ClockData::new(120.0, 96));

        let bpm = new_shrptr!(bpm::ClockBPMBinding(b.clone()));
        let ppq = new_shrptr!(bpm::ClockPPQBinding(b.clone()));
        let micros = new_shrptr!(bpm::ClockPeriodMicroBinding(b.clone()));
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
