use crate::Float;

const PERIOD_MICRO_MIN: Float = 0.001;
const BPM_MIN: Float = 0.001;
const PPQ_MIN: usize = 1;

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

impl ClockData {
    pub fn period_micros(bpm: Float, ppq: usize) -> Float {
        period_micro!(bpm, ppq)
    }

    pub fn new(bpm: Float, ppq: usize) -> Self {
        assert!(bpm > BPM_MIN);
        assert!(ppq > PPQ_MIN);
        Self {
            bpm,
            period_micros: Self::period_micros(bpm, ppq),
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
        self.bpm = num_traits::clamp(bpm, BPM_MIN, Float::MAX);
        self.period_micros = Self::period_micros(self.bpm, self.ppq);
    }

    fn period_micros(&self) -> Float {
        self.period_micros
    }

    fn set_period_micros(&mut self, period_micros: Float) {
        self.period_micros = num_traits::clamp(period_micros, PERIOD_MICRO_MIN, Float::MAX);
        self.bpm = 60.0e6 / (self.period_micros * self.ppq as Float);
    }

    fn ppq(&self) -> usize {
        self.ppq
    }

    fn set_ppq(&mut self, ppq: usize) {
        self.ppq = num_traits::clamp(ppq, PPQ_MIN, usize::MAX);
        self.period_micros = Self::period_micros(self.bpm, self.ppq);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bpm_value_test() {
        assert_eq!(5208f64, ClockData::period_micros(120.0, 96).floor());
        assert_eq!(20833f64, ClockData::period_micros(120.0, 24).floor());

        let mut c = ClockData::new(120.0, 96);
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
}
