extern crate euclidian_rythms;

use binding::bpm;
use binding::{BindingGetP, ParamBindingGet, ParamBindingSet, SpinlockParamBinding};
use context::{ChildContext, SchedContext};
use graph::{AChildP, ChildList, FuncWrapper, GraphExec, RootClock};
use midi::{MidiValue, NoteTrigger};
use std::net::{SocketAddrV4, UdpSocket};
use std::str::FromStr;
use std::sync::mpsc::sync_channel;
use std::sync::Arc;
use std::thread;
use util::Clamp;
use {LList, LNode, Sched, Scheduler, TimeResched, TimeSched};

pub struct Euclid {
    children: ChildList,
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
            children: LList::new(),
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
    fn exec(&mut self, context: &mut dyn SchedContext) -> bool {
        let step_ticks = self.step_ticks.get();

        if step_ticks > 0 && context.context_tick() % step_ticks == 0 {
            let steps = self.steps.get().clamp(0, 64);
            let pulses = self.pulses.get().clamp(0, 64);
            if steps > 0 && pulses > 0 {
                self.update_if(steps, pulses);

                //passing context through, so this is more like gate than a clock..
                let index = (context.context_tick() / step_ticks) % steps as usize;
                if self.pattern[index] {
                    for c in self.children.iter() {
                        c.lock().exec(context);
                    }
                }
            }
        }
        true
    }

    fn child_append(&mut self, child: AChildP) {
        self.children.push_back(child);
    }
}
