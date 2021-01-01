use crate::{
    binding::ParamBindingGet,
    context::ChildContext,
    event::EventEvalContext,
    graph::{GraphChildExec, GraphRootExec},
    tick::TickResched,
    Float,
};

#[cfg(not(feature = "std"))]
use num::traits::float::FloatCore;

/// A event_eval schedulable item that executes a graph tree.
pub struct RootClock<PeriodMicros>
where
    PeriodMicros: ParamBindingGet<Float>,
{
    tick: usize,
    tick_sub: Float,
    period_micros: PeriodMicros,
}

impl<PeriodMicros> RootClock<PeriodMicros>
where
    PeriodMicros: ParamBindingGet<Float>,
{
    pub fn new(period_micros: PeriodMicros) -> Self {
        Self {
            tick: 0,
            tick_sub: 0.0,
            period_micros,
        }
    }
}

impl<PeriodMicros> GraphRootExec for RootClock<PeriodMicros>
where
    PeriodMicros: ParamBindingGet<Float>,
{
    fn event_eval(
        &mut self,
        context: &mut dyn EventEvalContext,
        children: &mut dyn GraphChildExec,
    ) -> TickResched {
        let period_micros = self.period_micros.get();
        let mut ccontext = ChildContext::new(context, 0, self.tick, period_micros);
        children.child_exec_all(&mut ccontext);

        let ctp = context.context_tick_period_micros();
        if period_micros <= 0.0 || ctp <= 0.0 {
            TickResched::ContextRelative(1)
        } else {
            let next = self.tick_sub + (period_micros / ctp);
            self.tick_sub = next.fract();
            self.tick += 1;

            //XXX what if next is less than 1?
            //XXX could move root.node_exec in here execute multiple times..
            assert!(next >= 1.0, "tick less than sample size not supported");
            TickResched::ContextRelative(core::cmp::max(1, next.floor() as usize))
        }
    }
}
