use crate::{
    event::EventEvalContext,
    graph::{GraphChildExec, GraphNodeExec},
    param::ParamGet,
};

/// A step sequencer node
pub struct StepSeq<StepTicks, Index, const INDEX_CHILDREN: bool>
where
    StepTicks: ParamGet<usize>,
    Index: ParamGet<usize>,
{
    step_ticks: StepTicks,
    index: Index,
}

impl<StepTicks, Index, const INDEX_CHILDREN: bool> StepSeq<StepTicks, Index, INDEX_CHILDREN>
where
    StepTicks: ParamGet<usize>,
    Index: ParamGet<usize>,
{
    pub fn new(step_ticks: StepTicks, index: Index) -> Self {
        Self { step_ticks, index }
    }
}

impl<StepTicks, Index, E, const INDEX_CHILDREN: bool> GraphNodeExec<E>
    for StepSeq<StepTicks, Index, INDEX_CHILDREN>
where
    StepTicks: ParamGet<usize>,
    Index: ParamGet<usize>,
    E: Send,
{
    fn graph_exec(&self, context: &mut dyn EventEvalContext<E>, children: &dyn GraphChildExec<E>) {
        let step_ticks = self.step_ticks.get();

        if step_ticks > 0 && context.context_tick_now() % step_ticks == 0 {
            let index = self.index.get();
            if INDEX_CHILDREN {
                children.child_exec(context, index);
            } else {
                children.child_exec_all(context);
            }
        }
    }
}
