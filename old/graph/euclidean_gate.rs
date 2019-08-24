use crate::binding::ParamBindingGet;
use crate::context::SchedContext;
use crate::graph::{ChildCount, ChildExec, GraphExec};

pub struct Euclid<StepTicks, Steps, Pulses> {
    step_ticks: StepTicks,
    steps: Steps,
    pulses: Pulses,
    steps_last: Option<u8>,
    pulses_last: Option<u8>,
    pattern: [bool; 64],
}

impl<StepTicks, Steps, Pulses> Euclid<StepTicks, Steps, Pulses> {
    pub fn new(step_ticks: StepTicks, steps: Steps, pulses: Pulses) -> Self {
        Self {
            step_ticks,
            steps,
            pulses,
            steps_last: None,
            pulses_last: None,
            pattern: [false; 64],
        }
    }

    fn update_if(&mut self, steps: u8, pulses: u8) {
        if self.steps_last.is_some()
            && self.steps_last.unwrap() == steps
            && self.pulses_last.unwrap() == pulses
        {
            return;
        }
        self.steps_last = Some(steps);
        self.pulses_last = Some(pulses);

        euclidian_rythms::euclidian_rythm(&mut self.pattern, pulses as usize, steps as usize);
    }
}

impl<StepTicks, Steps, Pulses> GraphExec for Euclid<StepTicks, Steps, Pulses>
where
    StepTicks: ParamBindingGet<usize>,
    Steps: ParamBindingGet<u8>,
    Pulses: ParamBindingGet<u8>,
{
    fn exec(&mut self, context: &mut dyn SchedContext, children: &mut dyn ChildExec) -> bool {
        let step_ticks = self.step_ticks.get();

        if step_ticks > 0 && context.context_tick() % step_ticks == 0 {
            let steps = num::clamp(self.steps.get(), 0, 64);
            let pulses = num::clamp(self.pulses.get(), 0, 64);
            if steps > 0 && pulses > 0 {
                self.update_if(steps, pulses);

                //passing context through, so this is more like gate than a clock..
                let index = (context.context_tick() / step_ticks) % steps as usize;
                if self.pattern[index] {
                    children.exec_all(context);
                }
            }
        }
        //remove self if we have no children
        children.has_children()
    }

    fn children_max(&self) -> ChildCount {
        ChildCount::Inf
    }
}
