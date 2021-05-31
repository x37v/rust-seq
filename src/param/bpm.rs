use super::*;
use crate::{
    clock::{Clock, ClockData},
    Float,
};

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

/*
#[cfg(test)]
mod tests {
    use super::*;

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
*/
