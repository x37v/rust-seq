extern crate jack;
extern crate sched;

use sched::binding::bpm;
use sched::binding::ParamBinding;
use sched::context::{ChildContext, SchedContext};
use sched::graph::{ChildList, FuncWrapper, GraphExec, RootClock};
use sched::spinlock;
use sched::{LNode, Sched, Scheduler, TimeResched, TimeSched};
use std::sync::Arc;

use std::io;

fn main() {
    let (client, _status) =
        jack::Client::new("xnor_sched", jack::ClientOptions::NO_START_SERVER).unwrap();

    let mut midi_out = client
        .register_port("midi", jack::MidiOut::default())
        .unwrap();

    let mut s = Scheduler::new();
    s.spawn_helper_threads();

    let b = Arc::new(spinlock::Mutex::new(bpm::ClockData::new(120.0, 960)));
    let ppq = Arc::new(bpm::ClockPPQBinding(b.clone()));
    let micros = Arc::new(bpm::ClockPeriodMicroBinding(b.clone()));
    let mut clock = Box::new(RootClock::new(micros.clone()));

    let div = FuncWrapper::new_p(
        move |context: &mut dyn SchedContext, children: &mut ChildList| {
            let ppq_v = ppq.get();
            if context.context_tick() % ppq_v == 0 {
                let tick = context.context_tick() / ppq_v;
                let tick_period = context.base_tick_period_micros() * (ppq_v as f32);
                let mut ccontext = ChildContext::new(context, tick, tick_period);
                for c in children.iter() {
                    c.lock().exec(&mut ccontext);
                }
            }
            true
        },
    );

    let trig = FuncWrapper::new_p(
        move |context: &mut dyn SchedContext, _childen: &mut ChildList| {
            let index = context.context_tick();
            context.schedule_trigger(TimeSched::Relative(0), index);
            true
        },
    );

    div.lock().child_append(LNode::new_boxed(trig));
    clock.child_append(LNode::new_boxed(div));

    s.schedule(TimeSched::Relative(0), clock);

    /*
    s.schedule(
        TimeSched::Relative(0),
        Box::new(move |context: &mut dyn SchedContext| {
            context.schedule_trigger(TimeSched::Relative(0), 1);
            TimeResched::Relative(44100))
        },
    );
    */

    let mut e = s.executor().unwrap();
    let process_callback = move |client: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
        let mut out_p = midi_out.writer(ps);
        e.run(ps.n_frames() as usize, client.sample_rate() as usize);
        e.eval_triggers(&mut |time, index| {
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
