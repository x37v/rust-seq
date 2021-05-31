use super::*;
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

pub struct ClockGetPeriodMicro<P>
where
    P: ParamGet<ClockData>,
{
    clock: P,
}

pub struct ClockSetPeriodMicro<G, S>
where
    G: ParamGet<ClockData>,
    S: ParamSet<ClockData>,
{
    get: G,
    set: S,
}

pub struct ClockGetBPM<P>
where
    P: ParamGet<ClockData>,
{
    clock: P,
}

pub struct ClockSetBPM<G, S>
where
    G: ParamGet<ClockData>,
    S: ParamSet<ClockData>,
{
    get: G,
    set: S,
}

pub struct ClockGetPPQ<P>
where
    P: ParamGet<ClockData>,
{
    clock: P,
}

pub struct ClockSetPPQ<G, S>
where
    G: ParamGet<ClockData>,
    S: ParamSet<ClockData>,
{
    get: G,
    set: S,
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
        self.bpm = num_traits::clamp(bpm, BPM_MIN, Float::MAX);
        self.period_micros = Self::period_micro(self.bpm, self.ppq);
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
        self.period_micros = Self::period_micro(self.bpm, self.ppq);
    }
}

impl<P> ClockGetPeriodMicro<P>
where
    P: ParamGet<ClockData>,
{
    pub fn new(clock: P) -> Self {
        Self { clock }
    }
}

impl<P> ParamGet<Float> for ClockGetPeriodMicro<P>
where
    P: ParamGet<ClockData>,
{
    fn get(&self) -> Float {
        self.clock.get().period_micros()
    }
}

impl<G, S> ClockSetPeriodMicro<G, S>
where
    G: ParamGet<ClockData>,
    S: ParamSet<ClockData>,
{
    pub fn new(get: G, set: S) -> Self {
        Self { get, set }
    }
}

impl<G, S> ParamSet<Float> for ClockSetPeriodMicro<G, S>
where
    G: ParamGet<ClockData>,
    S: ParamSet<ClockData>,
{
    fn set(&self, period_micro: Float) {
        let mut clock = self.get.get();
        clock.set_period_micros(period_micro);
        self.set.set(clock);
    }
}

impl<P> ClockGetBPM<P>
where
    P: ParamGet<ClockData>,
{
    pub fn new(clock: P) -> Self {
        Self { clock }
    }
}

impl<P> ParamGet<Float> for ClockGetBPM<P>
where
    P: ParamGet<ClockData>,
{
    fn get(&self) -> Float {
        self.clock.get().bpm()
    }
}

impl<G, S> ClockSetBPM<G, S>
where
    G: ParamGet<ClockData>,
    S: ParamSet<ClockData>,
{
    pub fn new(get: G, set: S) -> Self {
        Self { get, set }
    }
}

impl<G, S> ParamSet<Float> for ClockSetBPM<G, S>
where
    G: ParamGet<ClockData>,
    S: ParamSet<ClockData>,
{
    fn set(&self, bpm: Float) {
        let mut clock = self.get.get();
        clock.set_bpm(bpm);
        self.set.set(clock);
    }
}

impl<P> ClockGetPPQ<P>
where
    P: ParamGet<ClockData>,
{
    pub fn new(clock: P) -> Self {
        Self { clock }
    }
}

impl<P> ParamGet<usize> for ClockGetPPQ<P>
where
    P: ParamGet<ClockData>,
{
    fn get(&self) -> usize {
        self.clock.get().ppq()
    }
}

impl<G, S> ClockSetPPQ<G, S>
where
    G: ParamGet<ClockData>,
    S: ParamSet<ClockData>,
{
    pub fn new(get: G, set: S) -> Self {
        Self { get, set }
    }
}

impl<G, S> ParamSet<usize> for ClockSetPPQ<G, S>
where
    G: ParamGet<ClockData>,
    S: ParamSet<ClockData>,
{
    fn set(&self, ppq: usize) {
        let mut clock = self.get.get();
        clock.set_ppq(ppq);
        self.set.set(clock);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    /*
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
    */
}
