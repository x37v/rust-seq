use crate::{
    clock::{Clock, ClockData},
    param::{ParamGet, ParamGetSet, ParamSet},
    Float,
};

pub struct ClockGetPeriodMicros<P>
where
    P: ParamGet<ClockData>,
{
    clock: P,
}

pub struct ClockSetPeriodMicros<G, S>
where
    G: ParamGet<ClockData>,
    S: ParamSet<ClockData>,
{
    get: G,
    set: S,
}

pub struct ClockGetSetPeriodMicros<P>
where
    P: ParamGet<ClockData> + ParamSet<ClockData>,
{
    param: ParamGetSet<ClockData, P>,
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

pub struct ClockGetSetBPM<P>
where
    P: ParamGet<ClockData> + ParamSet<ClockData>,
{
    param: ParamGetSet<ClockData, P>,
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

pub struct ClockGetSetPPQ<P>
where
    P: ParamGet<ClockData> + ParamSet<ClockData>,
{
    param: ParamGetSet<ClockData, P>,
}

impl<P> ClockGetPeriodMicros<P>
where
    P: ParamGet<ClockData>,
{
    pub fn new(clock: P) -> Self {
        Self { clock }
    }
}

impl<P> ParamGet<Float> for ClockGetPeriodMicros<P>
where
    P: ParamGet<ClockData>,
{
    fn get(&self) -> Float {
        self.clock.get().period_micros()
    }
}

impl<G, S> ClockSetPeriodMicros<G, S>
where
    G: ParamGet<ClockData>,
    S: ParamSet<ClockData>,
{
    pub fn new(get: G, set: S) -> Self {
        Self { get, set }
    }
}

impl<G, S> ParamSet<Float> for ClockSetPeriodMicros<G, S>
where
    G: ParamGet<ClockData>,
    S: ParamSet<ClockData>,
{
    fn set(&self, period_micros: Float) {
        let mut clock = self.get.get();
        clock.set_period_micros(period_micros);
        self.set.set(clock);
    }
}

impl<P> ClockGetSetPeriodMicros<P>
where
    P: ParamGet<ClockData> + ParamSet<ClockData>,
{
    pub fn new(param: P) -> Self {
        Self {
            param: ParamGetSet::new(param),
        }
    }
}

impl<P> ParamGet<Float> for ClockGetSetPeriodMicros<P>
where
    P: ParamGet<ClockData> + ParamSet<ClockData>,
{
    fn get(&self) -> Float {
        self.param.get().period_micros()
    }
}

impl<P> ParamSet<Float> for ClockGetSetPeriodMicros<P>
where
    P: ParamGet<ClockData> + ParamSet<ClockData>,
{
    fn set(&self, period_micros: Float) {
        let mut clock = self.param.get();
        clock.set_period_micros(period_micros);
        self.param.set(clock);
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

impl<P> ClockGetSetBPM<P>
where
    P: ParamGet<ClockData> + ParamSet<ClockData>,
{
    pub fn new(param: P) -> Self {
        Self {
            param: ParamGetSet::new(param),
        }
    }
}

impl<P> ParamGet<Float> for ClockGetSetBPM<P>
where
    P: ParamGet<ClockData> + ParamSet<ClockData>,
{
    fn get(&self) -> Float {
        self.param.get().bpm()
    }
}

impl<P> ParamSet<Float> for ClockGetSetBPM<P>
where
    P: ParamGet<ClockData> + ParamSet<ClockData>,
{
    fn set(&self, bpm: Float) {
        let mut clock = self.param.get();
        clock.set_bpm(bpm);
        self.param.set(clock);
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

impl<P> ClockGetSetPPQ<P>
where
    P: ParamGet<ClockData> + ParamSet<ClockData>,
{
    pub fn new(param: P) -> Self {
        Self {
            param: ParamGetSet::new(param),
        }
    }
}

impl<P> ParamGet<usize> for ClockGetSetPPQ<P>
where
    P: ParamGet<ClockData> + ParamSet<ClockData>,
{
    fn get(&self) -> usize {
        self.param.get().ppq()
    }
}

impl<P> ParamSet<usize> for ClockGetSetPPQ<P>
where
    P: ParamGet<ClockData> + ParamSet<ClockData>,
{
    fn set(&self, ppq: usize) {
        let mut clock = self.param.get();
        clock.set_ppq(ppq);
        self.param.set(clock);
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
