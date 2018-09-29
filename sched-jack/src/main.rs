extern crate euclidian_rythms;
extern crate jack;
extern crate sched;

use sched::binding::bpm;
use sched::binding::{BindingGetP, SpinlockParamBinding};
use sched::context::{ChildContext, SchedContext};
use sched::graph::{AChildP, ChildList, FuncWrapper, GraphExec, RootClock};
use sched::spinlock;
use sched::util::Clamp;
use sched::{LList, LNode, Sched, Scheduler, TimeSched};
use std::sync::Arc;

use std::io;

struct Euclid {
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

fn main() {
    let (client, _status) =
        jack::Client::new("xnor_sched", jack::ClientOptions::NO_START_SERVER).unwrap();

    let mut midi_out = client
        .register_port("midi", jack::MidiOut::default())
        .unwrap();

    let mut sched = Scheduler::new();
    sched.spawn_helper_threads();

    let bpm_binding = Arc::new(spinlock::Mutex::new(bpm::ClockData::new(120.0, 960)));
    let ppq = Arc::new(bpm::ClockPPQBinding(bpm_binding.clone()));
    let micros = Arc::new(bpm::ClockPeriodMicroBinding(bpm_binding.clone()));
    let mut clock = Box::new(RootClock::new(micros.clone()));

    let pulses = SpinlockParamBinding::new_p(2);
    let steps = SpinlockParamBinding::new_p(7);
    let step_ticks = SpinlockParamBinding::new_p(960 / 4);
    let euclid = Arc::new(spinlock::Mutex::new(Euclid::new(
        step_ticks.clone(),
        steps.clone(),
        pulses.clone(),
    )));

    /*
    let mut ppqc = ppq.clone();
    let div = FuncWrapper::new_p(
        move |context: &mut dyn SchedContext, children: &mut ChildList| {
            let div = ppqc.get();
            if context.context_tick() % div == 0 {
                let tick = context.context_tick() / div;
                let tick_period = context.base_tick_period_micros() * (div as f32);
                let mut ccontext = ChildContext::new(context, tick, tick_period);
                for c in children.iter() {
                    c.lock().exec(&mut ccontext);
                }
            }
            true
        },
    );
    */

    let trig = FuncWrapper::new_p(
        move |context: &mut dyn SchedContext, _childen: &mut ChildList| {
            let index = 0;
            context.schedule_trigger(TimeSched::Relative(0), index);
            true
        },
    );

    euclid.lock().child_append(LNode::new_boxed(trig));
    clock.child_append(LNode::new_boxed(euclid));

    sched.schedule(TimeSched::Relative(0), clock);

    let mut ex = sched.executor().unwrap();
    let process_callback = move |client: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
        let mut out_p = midi_out.writer(ps);
        ex.run(ps.n_frames() as usize, client.sample_rate() as usize);
        ex.eval_triggers(&mut |time, index| {
            let n = (index & 0x7F) as u8;
            let t = time as u32 % ps.n_frames();
            if out_p
                .write(&jack::RawMidi {
                    time: t,
                    bytes: &[0b1001_0000, n, 127],
                }).is_ok()
            {
                let _ = out_p.write(&jack::RawMidi {
                    time: t + 1,
                    bytes: &[0b1000_0000, n, 127],
                });
            }
        });
        jack::Control::Continue
    };

    let process = jack::ClosureProcessHandler::new(process_callback);

    // Activate the client, which starts the processing.
    let active_client = client.activate_async(Notifications, process).unwrap();

    // Wait for user input to quit
    println!("Press enter/return to quit...");
    let mut user_input = String::new();
    io::stdin().read_line(&mut user_input).ok();

    active_client.deactivate().unwrap();
}

struct Notifications;

impl jack::NotificationHandler for Notifications {
    fn thread_init(&self, _: &jack::Client) {
        println!("JACK: thread init");
    }

    fn shutdown(&mut self, status: jack::ClientStatus, reason: &str) {
        println!(
            "JACK: shutdown with status {:?} because \"{}\"",
            status, reason
        );
    }

    fn freewheel(&mut self, _: &jack::Client, is_enabled: bool) {
        println!(
            "JACK: freewheel mode is {}",
            if is_enabled { "on" } else { "off" }
        );
    }

    fn buffer_size(&mut self, _: &jack::Client, sz: jack::Frames) -> jack::Control {
        println!("JACK: buffer size changed to {}", sz);
        jack::Control::Continue
    }

    fn sample_rate(&mut self, _: &jack::Client, srate: jack::Frames) -> jack::Control {
        println!("JACK: sample rate changed to {}", srate);
        jack::Control::Continue
    }

    fn client_registration(&mut self, _: &jack::Client, name: &str, is_reg: bool) {
        println!(
            "JACK: {} client with name \"{}\"",
            if is_reg { "registered" } else { "unregistered" },
            name
        );
    }

    fn port_registration(&mut self, _: &jack::Client, port_id: jack::PortId, is_reg: bool) {
        println!(
            "JACK: {} port with id {}",
            if is_reg { "registered" } else { "unregistered" },
            port_id
        );
    }

    fn port_rename(
        &mut self,
        _: &jack::Client,
        port_id: jack::PortId,
        old_name: &str,
        new_name: &str,
    ) -> jack::Control {
        println!(
            "JACK: port with id {} renamed from {} to {}",
            port_id, old_name, new_name
        );
        jack::Control::Continue
    }

    fn ports_connected(
        &mut self,
        _: &jack::Client,
        port_id_a: jack::PortId,
        port_id_b: jack::PortId,
        are_connected: bool,
    ) {
        println!(
            "JACK: ports with id {} and {} are {}",
            port_id_a,
            port_id_b,
            if are_connected {
                "connected"
            } else {
                "disconnected"
            }
        );
    }

    fn graph_reorder(&mut self, _: &jack::Client) -> jack::Control {
        println!("JACK: graph reordered");
        jack::Control::Continue
    }

    fn xrun(&mut self, _: &jack::Client) -> jack::Control {
        println!("JACK: xrun occurred");
        jack::Control::Continue
    }

    fn latency(&mut self, _: &jack::Client, mode: jack::LatencyType) {
        println!(
            "JACK: {} latency has changed",
            match mode {
                jack::LatencyType::Capture => "capture",
                jack::LatencyType::Playback => "playback",
            }
        );
    }
}
