use crate::binding::ParamBindingGet;
use crate::context::ChildContext;
use crate::event::{EventEval, EventEvalContext};
use crate::graph::GraphNode;
use crate::tick::TickResched;

#[cfg(not(feature = "std"))]
use num::traits::float::FloatCore;

pub type Micro = f32;

/// A event_eval schedulable item that holds and executes a graph tree root.
pub struct RootClock<PeriodMicros, T>
where
    PeriodMicros: ParamBindingGet<Micro>,
    T: GraphNode,
{
    tick: usize,
    tick_sub: f32,
    period_micros: PeriodMicros,
    root: T,
}

impl<PeriodMicros, T> RootClock<PeriodMicros, T>
where
    PeriodMicros: ParamBindingGet<Micro>,
    T: GraphNode,
{
    pub fn new(period_micros: PeriodMicros, root: T) -> Self {
        Self {
            tick: 0,
            tick_sub: 0f32,
            period_micros,
            root,
        }
    }
}

impl<PeriodMicros, T> EventEval for RootClock<PeriodMicros, T>
where
    PeriodMicros: ParamBindingGet<Micro>,
    T: GraphNode,
{
    fn event_eval(&mut self, context: &mut dyn EventEvalContext) -> TickResched {
        let period_micros = self.period_micros.get();
        let mut ccontext = ChildContext::new(context, 0, self.tick, period_micros);
        self.root.node_exec(&mut ccontext);

        let ctp = context.context_tick_period_micros();
        if period_micros <= 0f32 || ctp <= 0f32 {
            TickResched::ContextRelative(1)
        } else {
            let next = self.tick_sub + (period_micros / ctp);
            self.tick_sub = next.fract();
            self.tick += 1;

            //XXX what if next is less than 1?
            //XXX could move root.node_exec in here execute multiple times..
            assert!(next >= 1f32, "tick less than sample size not supported");
            TickResched::ContextRelative(core::cmp::max(1, next.floor() as usize))
        }
    }
}
