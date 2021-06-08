use crate::{
    context::ChildContext,
    event::EventEvalContext,
    graph::{root::GraphRootExec, GraphChildExec},
    param::{ParamGet, ParamSet},
    tick::TickResched,
    Float,
};

#[cfg(not(feature = "std"))]
use num_traits::float::FloatCore;

//XXX tick must be get set because it has to be atomic

/// A root of a graph tree that evaluates its children at an interval controlled by its
/// period_micros `ParamGet`.
pub struct RootClock<P, TG, TS, SG, SS, R, E> {
    pub(crate) tick_get: TG,
    pub(crate) tick_set: TS,
    pub(crate) tick_sub_get: SG,
    pub(crate) tick_sub_set: SS,
    pub(crate) period_micros: P,
    pub(crate) run: R,
    pub(crate) _phantom: core::marker::PhantomData<E>,
}

impl<P, TG, TS, SG, SS, R, E> RootClock<P, TG, TS, SG, SS, R, E>
where
    P: ParamGet<Float>,
    TG: ParamGet<usize>,
    TS: ParamSet<usize>,
    SG: ParamGet<Float>,
    SS: ParamSet<Float>,
    R: ParamGet<bool>,
{
    pub fn new(
        period_micros: P,
        tick_get: TG,
        tick_set: TS,
        tick_sub_get: SG,
        tick_sub_set: SS,
        run: R,
    ) -> Self {
        Self {
            tick_get,
            tick_set,
            tick_sub_get,
            tick_sub_set,
            period_micros,
            run,
            _phantom: Default::default(),
        }
    }
}

impl<P, TG, TS, SG, SS, R, E> GraphRootExec<E> for RootClock<P, TG, TS, SG, SS, R, E>
where
    P: ParamGet<Float>,
    TG: ParamGet<usize>,
    TS: ParamSet<usize>,
    SG: ParamGet<Float>,
    SS: ParamSet<Float>,
    R: ParamGet<bool>,
    E: Send,
{
    fn event_eval(
        &mut self,
        context: &mut dyn EventEvalContext<E>,
        children: &mut dyn GraphChildExec<E>,
    ) -> TickResched {
        if self.run.get() {
            let period_micros = self.period_micros.get();
            let tick = self.tick_get.get();
            let mut ccontext = ChildContext::new(context, 0, tick, period_micros);
            children.child_exec_all(&mut ccontext);

            let ctp = context.context_tick_period_micros();
            if period_micros <= 0.0 || ctp <= 0.0 {
                TickResched::ContextRelative(1)
            } else {
                let next = self.tick_sub_get.get() + (period_micros / ctp);
                self.tick_sub_set.set(next.fract());
                self.tick_set.set(tick + 1);

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
