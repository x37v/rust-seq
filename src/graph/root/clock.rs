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
pub struct RootClock<P, R, RS, E, U> {
    pub(crate) tick: usize,
    pub(crate) tick_sub: Float,
    pub(crate) period_micros: P,
    pub(crate) run: R,
    pub(crate) reset: RS,
    pub(crate) _phantom: core::marker::PhantomData<(E, U)>,
}

impl<P, R, RS, E, U> RootClock<P, R, RS, E, U>
where
    P: ParamGet<Float, U>,
    R: ParamGet<bool, U>,
    RS: ParamGet<bool, U>,
    R: ParamGet<bool, U>,
{
    pub fn new(period_micros: P, run: R, reset: RS) -> Self {
        Self {
            tick: 0,
            tick_sub: 0.0 as Float,
            period_micros,
            run,
            reset,
            _phantom: Default::default(),
        }
    }
}

impl<P, R, RS, E, U> GraphRootExec<E, U> for RootClock<P, R, RS, E, U>
where
    P: ParamGet<Float, U>,
    R: ParamGet<bool, U>,
    RS: ParamGet<bool, U>,
{
    fn event_eval(
        &mut self,
        context: &mut dyn EventEvalContext<E>,
        user_data: &mut U,
        children: &mut dyn GraphChildExec<E, U>,
    ) -> TickResched {
        if self.run.get(user_data) {
            let period_micros = self.period_micros.get(user_data);
            let (tick, tick_sub) = if self.reset.get(user_data) {
                (0, 0.0)
            } else {
                (self.tick, self.tick_sub)
            };
            let mut ccontext = ChildContext::new(context, 0, tick, period_micros);
            children.child_exec_all(&mut ccontext, user_data);

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
