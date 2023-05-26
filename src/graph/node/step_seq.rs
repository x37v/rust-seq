use crate::{
    event::EventEvalContext,
    graph::{GraphChildExec, GraphNodeExec},
    param::ParamGet,
};

/// A step sequencer node
pub struct StepSeq<StepTicks, Index, U, const INDEX_CHILDREN: bool>
where
    StepTicks: ParamGet<usize, U>,
    Index: ParamGet<usize, U>,
{
    step_ticks: StepTicks,
    index: Index,
    _phantom: core::marker::PhantomData<U>,
}

impl<StepTicks, Index, U, const INDEX_CHILDREN: bool> StepSeq<StepTicks, Index, U, INDEX_CHILDREN>
where
    StepTicks: ParamGet<usize, U>,
    Index: ParamGet<usize, U>,
{
    pub fn new(step_ticks: StepTicks, index: Index) -> Self {
        Self {
            step_ticks,
            index,
            _phantom: Default::default(),
        }
    }
}

impl<StepTicks, Index, E, U, const INDEX_CHILDREN: bool> GraphNodeExec<E, U>
    for StepSeq<StepTicks, Index, U, INDEX_CHILDREN>
where
    U: Send,
    StepTicks: ParamGet<usize, U>,
    Index: ParamGet<usize, U>,
{
    fn graph_exec(
        &self,
        context: &mut dyn EventEvalContext<E>,
        children: &dyn GraphChildExec<E, U>,
        user_data: &mut U,
    ) {
        let step_ticks = self.step_ticks.get(user_data);

        if step_ticks > 0 && context.context_tick_now() % step_ticks == 0 {
            let index = self.index.get(user_data);
            if INDEX_CHILDREN {
                children.child_exec(context, index, user_data);
            } else {
                children.child_exec_all(context, user_data);
            }
        }
    }
}
