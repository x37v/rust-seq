use crate::{
    context::ChildContext,
    event::EventEvalContext,
    graph::{GraphChildExec, GraphNodeExec},
    param::ParamGet,
    tick::offset_tick,
};

//only applies offset to context
pub struct TickOffset<T> {
    offset: T,
}

impl<T> TickOffset<T>
where
    T: ParamGet<isize>,
{
    pub fn new(offset: T) -> Self {
        Self { offset }
    }
}

impl<T, E> GraphNodeExec<E> for TickOffset<T>
where
    T: ParamGet<isize>,
{
    fn graph_exec(&self, context: &mut dyn EventEvalContext<E>, children: &dyn GraphChildExec<E>) {
        let period_micros = context.context_tick_period_micros();
        let context_tick = offset_tick(context.context_tick_now(), self.offset.get());

        let mut ccontext = ChildContext::new(context, 0, context_tick, period_micros);
        children.child_exec_all(&mut ccontext);
    }
}
