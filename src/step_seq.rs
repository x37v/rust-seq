use binding::BindingGetP;
use context::SchedContext;
use graph::{ChildCount, ChildExec, GraphExec};

pub struct StepSeq {
    step_ticks: BindingGetP<usize>,
    steps: BindingGetP<usize>,
}

impl StepSeq {
    pub fn new(step_ticks: BindingGetP<usize>, steps: BindingGetP<usize>) -> Self {
        Self { step_ticks, steps }
    }
}

//step sequencer acts as a gate, triggering its appropriate child with the context passed in only
//at step_ticks context ticks
impl GraphExec for StepSeq {
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
