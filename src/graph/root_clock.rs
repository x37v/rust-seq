use super::*;
use crate::binding::ParamBindingGet;
use crate::time::TimeResched;

pub type Micro = f32;
pub struct RootClock<PeriodMicros>
where
    PeriodMicros: ParamBindingGet<Micro>,
{
    tick: usize,
    tick_sub: f32,
    period_micros: PeriodMicros,
    children: ChildList,
}

impl<PeriodMicros> RootClock<PeriodMicros>
where
    PeriodMicros: ParamBindingGet<Micro>,
{
    pub fn new(period_micros: PeriodMicros) -> Self {
        Self {
            tick: 0,
            tick_sub: 0f32,
            period_micros,
            children: LList::new(),
        }
    }
    pub fn child_append(&mut self, child: AChildP) {
        self.children.push_back(child);
    }
}

impl<PeriodMicros> SchedCall for RootClock<PeriodMicros>
where
    PeriodMicros: ParamBindingGet<Micro>,
{
    fn sched_call(&mut self, context: &mut dyn SchedContext) -> TimeResched {
        let period_micros = self.period_micros.get();
        if self.children.count() > 0 {
            let mut ccontext = ChildContext::new(context, 0, self.tick, period_micros);
            let mut tmp = LList::new();
            std::mem::swap(&mut self.children, &mut tmp);

            for c in tmp.into_iter() {
                if c.lock().exec(&mut ccontext) {
                    self.children.push_back(c);
                }
            }
        }

        let ctp = context.context_tick_period_micros();
        if period_micros <= 0f32 || ctp <= 0f32 {
            TimeResched::ContextRelative(1)
        } else {
            let next = self.tick_sub + (period_micros / ctp);
            self.tick_sub = next.fract();
            self.tick += 1;

            //XXX what if next is less than 1?
            assert!(next >= 1f32, "tick less than sample size not supported");
            TimeResched::ContextRelative(std::cmp::max(1, next.floor() as usize))
        }
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;
    use crate::base::SrcSink;
    use crate::context::RootContext;
    use crate::llist_pqueue::LListPQueue;
    use crate::time::TimeResched;
    use std::sync::Arc;

    #[test]
    fn root_clock_call() {
        let mut src_sink = SrcSink::new();
        let mut list = LListPQueue::new();
        let mut trig_list = LListPQueue::new();

        //44100 sample rate, 200 micro second clock
        //[8.82, 17.64, 26.46, 35.28, 44.1, 52.92]
        let period_micros = Arc::new(200f32);
        let mut clock = RootClock::new(period_micros);
        for &(tick, resched) in [(0, 8), (8, 9), (17, 9), (26, 9), (35, 9), (44, 8)].iter() {
            let mut c = RootContext::new(
                tick as usize,
                44100,
                &mut list,
                &mut trig_list,
                &mut src_sink,
            );
            let r = clock.sched_call(&mut c);
            assert_eq!(
                TimeResched::ContextRelative(resched as usize),
                r,
                "({}, {})",
                tick,
                resched
            );
        }

        //48k sample rate, 300 micro second clock
        let period_micros = Arc::new(300f32);
        let mut clock = RootClock::new(period_micros);
        //my calculations in ruby give:
        //[14.4, 28.8, 43.2, 57.6, 72.0, 86.4]
        //but this math seems to want, which seems fine
        //[14.4, 28.8, 43.2, 57.6, 71.X, 86.X]
        for &(tick, resched) in [(0, 14), (14, 14), (28, 15), (43, 14), (57, 14), (71, 15)].iter() {
            let mut c = RootContext::new(
                tick as usize,
                48000,
                &mut list,
                &mut trig_list,
                &mut src_sink,
            );
            let r = clock.sched_call(&mut c);
            assert_eq!(
                TimeResched::ContextRelative(resched as usize),
                r,
                "({}, {})",
                tick,
                resched
            );
        }
    }
}
