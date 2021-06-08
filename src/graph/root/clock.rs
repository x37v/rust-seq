use crate::{
    context::ChildContext,
    event::EventEvalContext,
    graph::{root::GraphRootExec, GraphChildExec},
    param::ParamGet,
    tick::TickResched,
    Float,
};

#[cfg(not(feature = "std"))]
use num_traits::float::FloatCore;

/// A root of a graph tree that evaluates its children at an interval controlled by its
/// period_micros `ParamGet`.
pub struct RootClock<P, R, RS, E> {
    pub(crate) tick: usize,
    pub(crate) tick_sub: Float,
    pub(crate) period_micros: P,
    pub(crate) run: R,
    pub(crate) reset: RS,
    pub(crate) _phantom: core::marker::PhantomData<E>,
}

impl<P, R, RS, E> RootClock<P, R, RS, E>
where
    P: ParamGet<Float>,
    R: ParamGet<bool>,
    RS: ParamGet<bool>,
    R: ParamGet<bool>,
{
    pub fn new(period_micros: P, tick: usize, tick_sub: Float, run: R, reset: RS) -> Self {
        Self {
            tick,
            tick_sub,
            period_micros,
            run,
            reset,
            _phantom: Default::default(),
        }
    }
}

impl<P, R, RS, E> GraphRootExec<E> for RootClock<P, R, RS, E>
where
    P: ParamGet<Float>,
    R: ParamGet<bool>,
    RS: ParamGet<bool>,
    E: Send,
{
    fn event_eval(
        &mut self,
        context: &mut dyn EventEvalContext<E>,
        children: &mut dyn GraphChildExec<E>,
    ) -> TickResched {
        if self.run.get() {
            let period_micros = self.period_micros.get();
            let (tick, tick_sub) = if self.reset.get() {
                (0, 0.0)
            } else {
                (self.tick, self.tick_sub)
            };
            let mut ccontext = ChildContext::new(context, 0, tick, period_micros);
            children.child_exec_all(&mut ccontext);

            let ctp = context.context_tick_period_micros();
            if period_micros <= 0.0 || ctp <= 0.0 {
                TickResched::ContextRelative(1)
            } else {
                let next = tick_sub + (period_micros / ctp);
                self.tick_sub = next.fract();
                self.tick = tick + 1;

                //XXX what if next is less than 1?
                //XXX could move root.node_exec in here execute multiple times..
                assert!(next >= 1.0, "tick less than sample size not supported");
                TickResched::ContextRelative(core::cmp::max(1, next.floor() as usize))
            }
        } else {
            TickResched::ContextRelative(1)
        }
    }
}
