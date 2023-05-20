use crate::{
    context::ChildContext,
    event::EventEvalContext,
    graph::{GraphChildExec, GraphNodeExec},
    param::ParamGet,
    tick::offset_tick,
};

//only applies offset to context
pub struct TickOffset<T, U> {
    offset: T,
    _phatom: core::marker::PhantomData<U>,
}

impl<T, U> TickOffset<T, U>
where
    T: ParamGet<isize, U>,
{
    pub fn new(offset: T) -> Self {
        Self {
            offset,
            _phatom: Default::default(),
        }
    }
}

impl<T, E, U> GraphNodeExec<E, U> for TickOffset<T, U>
where
    T: ParamGet<isize, U>,
{
    fn graph_exec(
        &self,
        context: &mut dyn EventEvalContext<E>,
        children: &dyn GraphChildExec<E, U>,
        user_data: &mut U,
    ) {
        let period_micros = context.context_tick_period_micros();
        let context_tick = offset_tick(context.context_tick_now(), self.offset.get(user_data));

        let mut ccontext = ChildContext::new(context, 0, context_tick, period_micros);
        children.child_exec_all(&mut ccontext, user_data);
    }
}
