use crate::binding::ParamBindingGet;
use crate::event::EventEvalContext;
use crate::graph::{GraphChildExec, GraphNodeExec};
use core::ops::Deref;

pub struct StepSeq<StepTicks, Steps>
where
    StepTicks: Deref<Target = dyn ParamBindingGet<usize>> + Send + Sync,
    Steps: Deref<Target = dyn ParamBindingGet<usize>> + Send + Sync,
{
    step_ticks: StepTicks,
    steps: Steps,
}

impl<StepTicks, Steps> StepSeq<StepTicks, Steps>
where
    StepTicks: Deref<Target = dyn ParamBindingGet<usize>> + Send + Sync,
    Steps: Deref<Target = dyn ParamBindingGet<usize>> + Send + Sync,
{
    pub fn new(step_ticks: StepTicks, steps: Steps) -> Self {
        Self { step_ticks, steps }
    }
}

//step sequencer acts as a gate, triggering its appropriate child with the context passed in only
//at step_ticks context ticks
impl<StepTicks, Steps> GraphNodeExec for StepSeq<StepTicks, Steps>
where
    StepTicks: Deref<Target = dyn ParamBindingGet<usize>> + Send + Sync,
    Steps: Deref<Target = dyn ParamBindingGet<usize>> + Send + Sync,
{
    fn graph_exec(
        &mut self,
        context: &mut dyn EventEvalContext,
        children: &mut dyn GraphChildExec,
    ) {
        let step_ticks = self.step_ticks.get();

        if step_ticks > 0 && context.context_tick_now() % step_ticks == 0 {
            let steps = self.steps.get();
            if steps > 0 {
                let index = (context.context_tick_now() / step_ticks) % steps as usize;
                children.child_exec(context, index);
            }
        }
    }
}
