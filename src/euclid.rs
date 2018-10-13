extern crate euclidian_rythms;

use binding::BindingGetP;
use context::SchedContext;
use graph::{ChildCount, ChildExec, GraphExec};
use util::Clamp;

pub struct Euclid {
    step_ticks: BindingGetP<usize>,
    steps: BindingGetP<u8>,
    pulses: BindingGetP<u8>,
    steps_last: Option<u8>,
    pulses_last: Option<u8>,
    pattern: [bool; 64],
}

impl Euclid {
    pub fn new(
        step_ticks: BindingGetP<usize>,
        steps: BindingGetP<u8>,
        pulses: BindingGetP<u8>,
    ) -> Self {
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

impl GraphExec for Euclid {
    fn exec(&mut self, context: &mut dyn SchedContext, children: &mut dyn ChildExec) -> bool {
        let step_ticks = self.step_ticks.get();

        if step_ticks > 0 && context.context_tick() % step_ticks == 0 {
            let steps = self.steps.get().clamp(0, 64);
            let pulses = self.pulses.get().clamp(0, 64);
            if steps > 0 && pulses > 0 {
                self.update_if(steps, pulses);

                //passing context through, so this is more like gate than a clock..
                let index = (context.context_tick() / step_ticks) % steps as usize;
                if self.pattern[index] {
                    children.exec_all(context);
                }
            }
        }
        true
    }

    fn allowed_children(&self) -> ChildCount {
        ChildCount::Inf
    }
}
