use crate::{
    context::ChildContext,
    event::EventEvalContext,
    graph::{root::GraphRootExec, GraphChildExec},
    param::ParamGet,
    tick::TickResched,
    Float,
};

#[cfg(not(test))]
use num_traits::float::FloatCore;

/// A root of a graph tree that evaluates its children at an interval controlled by its
/// period_micros `ParamGet`.
pub struct RootClock<P, E>
where
    P: ParamGet<Float>,
{
    pub(crate) tick: usize,
    pub(crate) tick_sub: Float,
    pub(crate) period_micros: P,
    pub(crate) _phantom: core::marker::PhantomData<E>,
}

impl<P, E> RootClock<P, E>
where
    P: ParamGet<Float>,
{
    pub fn new(period_micros: P) -> Self {
        Self {
            tick: 0,
            tick_sub: 0.0,
            period_micros,
            _phantom: Default::default(),
        }
    }
}

impl<P, E> GraphRootExec<E> for RootClock<P, E>
where
    P: ParamGet<Float>,
    E: Send,
{
    fn event_eval(
        &mut self,
        context: &mut dyn EventEvalContext<E>,
        children: &mut dyn GraphChildExec<E>,
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
