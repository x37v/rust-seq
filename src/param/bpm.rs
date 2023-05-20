use crate::{
    clock::{Clock, ClockData},
    param::{ParamGet, ParamGetSet, ParamSet},
    Float,
};

pub struct ClockGetPeriodMicros<P, U>
where
    P: ParamGet<ClockData, U>,
{
    clock: P,
    _phantom: core::marker::PhantomData<U>,
}

pub struct ClockSetPeriodMicros<G, S, U>
where
    G: ParamGet<ClockData, U>,
    S: ParamSet<ClockData, U>,
{
    get: G,
    set: S,
    _phantom: core::marker::PhantomData<U>,
}

pub struct ClockGetSetPeriodMicros<P, U>
where
    P: ParamGet<ClockData, U> + ParamSet<ClockData, U>,
{
    param: ParamGetSet<ClockData, P, U>,
}

pub struct ClockGetBPM<P, U>
where
    P: ParamGet<ClockData, U>,
{
    clock: P,
    _phantom: core::marker::PhantomData<U>,
}

pub struct ClockSetBPM<G, S, U>
where
    G: ParamGet<ClockData, U>,
    S: ParamSet<ClockData, U>,
{
    get: G,
    set: S,
    _phantom: core::marker::PhantomData<U>,
}

pub struct ClockGetSetBPM<P, U>
where
    P: ParamGet<ClockData, U> + ParamSet<ClockData, U>,
{
    param: ParamGetSet<ClockData, P, U>,
}

pub struct ClockGetPPQ<P, U>
where
    P: ParamGet<ClockData, U>,
{
    clock: P,
    _phantom: core::marker::PhantomData<U>,
}

pub struct ClockSetPPQ<G, S, U>
where
    G: ParamGet<ClockData, U>,
    S: ParamSet<ClockData, U>,
{
    get: G,
    set: S,
    _phantom: core::marker::PhantomData<U>,
}

pub struct ClockGetSetPPQ<P, U>
where
    P: ParamGet<ClockData, U> + ParamSet<ClockData, U>,
{
    param: ParamGetSet<ClockData, P, U>,
}

impl<P, U> ClockGetPeriodMicros<P, U>
where
    P: ParamGet<ClockData, U>,
{
    pub fn new(clock: P) -> Self {
        Self {
            clock,
            _phantom: Default::default(),
        }
    }
}

impl<P, U> ParamGet<Float, U> for ClockGetPeriodMicros<P, U>
where
    P: ParamGet<ClockData, U>,
{
    fn get(&self, user_data: &mut U) -> Float {
        self.clock.get(user_data).period_micros()
    }
}

impl<G, S, U> ClockSetPeriodMicros<G, S, U>
where
    G: ParamGet<ClockData, U>,
    S: ParamSet<ClockData, U>,
{
    pub fn new(get: G, set: S) -> Self {
        Self {
            get,
            set,
            _phantom: Default::default(),
        }
    }
}

impl<G, S, U> ParamSet<Float, U> for ClockSetPeriodMicros<G, S, U>
where
    G: ParamGet<ClockData, U>,
    S: ParamSet<ClockData, U>,
{
    fn set(&self, period_micros: Float, user_data: &mut U) {
        let mut clock = self.get.get(user_data);
        clock.set_period_micros(period_micros);
        self.set.set(clock, user_data);
    }
}

impl<P, U> ClockGetSetPeriodMicros<P, U>
where
    P: ParamGet<ClockData, U> + ParamSet<ClockData, U>,
{
    pub fn new(param: P) -> Self {
        Self {
            param: ParamGetSet::new(param),
        }
    }
}

impl<P, U> ParamGet<Float, U> for ClockGetSetPeriodMicros<P, U>
where
    P: ParamGet<ClockData, U> + ParamSet<ClockData, U>,
{
    fn get(&self, user_data: &mut U) -> Float {
        self.param.get(user_data).period_micros()
    }
}

impl<P, U> ParamSet<Float, U> for ClockGetSetPeriodMicros<P, U>
where
    P: ParamGet<ClockData, U> + ParamSet<ClockData, U>,
{
    fn set(&self, period_micros: Float, user_data: &mut U) {
        let mut clock = self.param.get(user_data);
        clock.set_period_micros(period_micros);
        self.param.set(clock, user_data);
    }
}

impl<P, U> ClockGetBPM<P, U>
where
    P: ParamGet<ClockData, U>,
{
    pub fn new(clock: P) -> Self {
        Self {
            clock,
            _phantom: Default::default(),
        }
    }
}

impl<P, U> ParamGet<Float, U> for ClockGetBPM<P, U>
where
    P: ParamGet<ClockData, U>,
{
    fn get(&self, user_data: &mut U) -> Float {
        self.clock.get(user_data).bpm()
    }
}

impl<G, S, U> ClockSetBPM<G, S, U>
where
    G: ParamGet<ClockData, U>,
    S: ParamSet<ClockData, U>,
{
    pub fn new(get: G, set: S) -> Self {
        Self {
            get,
            set,
            _phantom: Default::default(),
        }
    }
}

impl<G, S, U> ParamSet<Float, U> for ClockSetBPM<G, S, U>
where
    G: ParamGet<ClockData, U>,
    S: ParamSet<ClockData, U>,
{
    fn set(&self, bpm: Float, user_data: &mut U) {
        let mut clock = self.get.get(user_data);
        clock.set_bpm(bpm);
        self.set.set(clock, user_data);
    }
}

impl<P, U> ClockGetSetBPM<P, U>
where
    P: ParamGet<ClockData, U> + ParamSet<ClockData, U>,
{
    pub fn new(param: P) -> Self {
        Self {
            param: ParamGetSet::new(param),
        }
    }
}

impl<P, U> ParamGet<Float, U> for ClockGetSetBPM<P, U>
where
    P: ParamGet<ClockData, U> + ParamSet<ClockData, U>,
{
    fn get(&self, user_data: &mut U) -> Float {
        self.param.get(user_data).bpm()
    }
}

impl<P, U> ParamSet<Float, U> for ClockGetSetBPM<P, U>
where
    P: ParamGet<ClockData, U> + ParamSet<ClockData, U>,
{
    fn set(&self, bpm: Float, user_data: &mut U) {
        let mut clock = self.param.get(user_data);
        clock.set_bpm(bpm);
        self.param.set(clock, user_data);
    }
}

impl<P, U> ClockGetPPQ<P, U>
where
    P: ParamGet<ClockData, U>,
{
    pub fn new(clock: P) -> Self {
        Self {
            clock,
            _phantom: Default::default(),
        }
    }
}

impl<P, U> ParamGet<usize, U> for ClockGetPPQ<P, U>
where
    P: ParamGet<ClockData, U>,
{
    fn get(&self, user_data: &mut U) -> usize {
        self.clock.get(user_data).ppq()
    }
}

impl<G, S, U> ClockSetPPQ<G, S, U>
where
    G: ParamGet<ClockData, U>,
    S: ParamSet<ClockData, U>,
{
    pub fn new(get: G, set: S) -> Self {
        Self {
            get,
            set,
            _phantom: Default::default(),
        }
    }
}

impl<G, S, U> ParamSet<usize, U> for ClockSetPPQ<G, S, U>
where
    G: ParamGet<ClockData, U>,
    S: ParamSet<ClockData, U>,
{
    fn set(&self, ppq: usize, user_data: &mut U) {
        let mut clock = self.get.get(user_data);
        clock.set_ppq(ppq);
        self.set.set(clock, user_data);
    }
}

impl<P, U> ClockGetSetPPQ<P, U>
where
    P: ParamGet<ClockData, U> + ParamSet<ClockData, U>,
{
    pub fn new(param: P) -> Self {
        Self {
            param: ParamGetSet::new(param),
        }
    }
}

impl<P, U> ParamGet<usize, U> for ClockGetSetPPQ<P, U>
where
    P: ParamGet<ClockData, U> + ParamSet<ClockData, U>,
{
    fn get(&self, user_data: &mut U) -> usize {
        self.param.get(user_data).ppq()
    }
}

impl<P, U> ParamSet<usize, U> for ClockGetSetPPQ<P, U>
where
    P: ParamGet<ClockData, U> + ParamSet<ClockData, U>,
{
    fn set(&self, ppq: usize, user_data: &mut U) {
        let mut clock = self.param.get(user_data);
        clock.set_ppq(ppq);
        self.param.set(clock, user_data);
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
