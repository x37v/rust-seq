use crate::binding::ParamBindingGet;
use crate::context::SchedContext;
use crate::graph::{ChildCount, ChildExec, GraphExec};

pub struct StepSeq<StepTicks, Steps>
where
    StepTicks: ParamBindingGet<usize>,
    Steps: ParamBindingGet<usize>,
{
    step_ticks: StepTicks,
    steps: Steps,
}

impl<StepTicks, Steps> StepSeq<StepTicks, Steps>
where
    StepTicks: ParamBindingGet<usize>,
    Steps: ParamBindingGet<usize>,
{
    pub fn new(step_ticks: StepTicks, steps: Steps) -> Self {
        Self { step_ticks, steps }
    }
}

//step sequencer acts as a gate, triggering its appropriate child with the context passed in only
//at step_ticks context ticks
impl<StepTicks, Steps> GraphExec for StepSeq<StepTicks, Steps>
where
    StepTicks: ParamBindingGet<usize>,
    Steps: ParamBindingGet<usize>,
{
    fn exec(&mut self, context: &mut dyn SchedContext, children: &mut dyn ChildExec) -> bool {
        let step_ticks = self.step_ticks.get();

        if step_ticks > 0 && context.context_tick() % step_ticks == 0 {
            let steps = self.steps.get();
            if steps > 0 {
                let index = (context.context_tick() / step_ticks) % steps as usize;
                children.exec(context, index);
            }
        }
        //remove self if we have no children
        children.has_children()
    }

    fn children_max(&self) -> ChildCount {
        ChildCount::Inf
    }
}
