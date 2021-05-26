use crate::{
    event::EventEvalContext,
    graph::{GraphChildExec, GraphNodeExec},
    param::{ParamGet, ParamSet},
};

/// A step sequencer node
pub struct StepSeq<StepTicks, Steps, Index, const INDEX_CHILDREN: bool>
where
    StepTicks: ParamGet<usize>,
    Steps: ParamGet<usize>,
    Index: ParamSet<usize>,
{
    step_ticks: StepTicks,
    steps: Steps,
    index: Index,
}

impl<StepTicks, Steps, Index, const INDEX_CHILDREN: bool>
    StepSeq<StepTicks, Steps, Index, INDEX_CHILDREN>
where
    StepTicks: ParamGet<usize>,
    Steps: ParamGet<usize>,
    Index: ParamSet<usize>,
{
    pub fn new(step_ticks: StepTicks, steps: Steps, index: Index) -> Self {
        Self {
            step_ticks,
            steps,
            index,
        }
    }
}

impl<StepTicks, Steps, Index, E, const INDEX_CHILDREN: bool> GraphNodeExec<E>
    for StepSeq<StepTicks, Steps, Index, INDEX_CHILDREN>
where
    StepTicks: ParamGet<usize>,
    Steps: ParamGet<usize>,
    Index: ParamSet<usize>,
    E: Send,
{
    fn graph_exec(&self, context: &mut dyn EventEvalContext<E>, children: &dyn GraphChildExec<E>) {
        let step_ticks = self.step_ticks.get();

        if step_ticks > 0 && context.context_tick_now() % step_ticks == 0 {
            let steps = self.steps.get();
            if steps > 0 {
                let index = (context.context_tick_now() / step_ticks) % steps as usize;
                self.index.set(index);
                if INDEX_CHILDREN {
                    children.child_exec(context, index);
                } else {
                    children.child_exec_all(context);
                }
            }
        }
    }
}
