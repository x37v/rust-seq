extern crate jack;
extern crate rosc;
extern crate sched;

use rosc::{OscPacket, OscType};
use sched::binding::bpm;
use sched::binding::{ParamBindingGet, ParamBindingSet, SpinlockParamBinding};
use sched::context::{ChildContext, SchedContext};
#[allow(unused_imports)]
use sched::euclid::Euclid;
use sched::graph::{
    ChildCount, ChildExec, FuncWrapper, GraphIndexExec, GraphNode, GraphNodeWrapper,
    IndexFuncWrapper, NChildGraphNodeWrapper, RootClock,
};
use sched::midi::{MidiValue, NoteTrigger};
use sched::spinlock;
use sched::step_seq::StepSeq;
use sched::{LNode, Sched, Scheduler, TimeResched, TimeSched};
use std::net::{SocketAddrV4, UdpSocket};
use std::str::FromStr;
use std::sync::mpsc::sync_channel;
use std::sync::Arc;
use std::thread;

use std::io;

fn main() {
    let (client, _status) =
        jack::Client::new("xnor_sched", jack::ClientOptions::NO_START_SERVER).unwrap();

    let mut midi_out = client
        .register_port("midi", jack::MidiOut::default())
        .unwrap();

    let midi_in = client
        .register_port("control", jack::MidiIn::default())
        .unwrap();

    let mut sched = Scheduler::new();
    sched.spawn_helper_threads();

    let (msender, mreceiver) = sync_channel(1024);
    let note_trig = Arc::new(spinlock::Mutex::new(NoteTrigger::new(0, msender)));

    let bpm_binding = Arc::new(spinlock::Mutex::new(bpm::ClockData::new(120.0, 960)));
    let bpm = Arc::new(bpm::ClockBPMBinding(bpm_binding.clone()));
    let ppq = Arc::new(bpm::ClockPPQBinding(bpm_binding.clone()));
    let micros = Arc::new(bpm::ClockPeriodMicroBinding(bpm_binding.clone()));
    let mut clock = Box::new(RootClock::new(micros.clone()));

    let pulses = SpinlockParamBinding::new_p(2);
    let steps = SpinlockParamBinding::new_p(16);
    let step_ticks = SpinlockParamBinding::new_p(960 / 4);

    //build up gates
    let gates: Vec<Arc<SpinlockParamBinding<bool>>> = vec![false; 16]
        .iter()
        .map(|v| SpinlockParamBinding::new_p(*v))
        .collect();
    let toggles = gates.clone();
    let step_gate = SpinlockParamBinding::new_p(false);

    let addr_s = "127.0.0.1:10001";
    let addr = match SocketAddrV4::from_str(addr_s) {
        Ok(addr) => addr,
        Err(e) => panic!("error with osc address {}", e),
    };
    /*
    println!("osc addr {}", addr_s);
    let _osc_thread = thread::spawn(move || {
        let sock = UdpSocket::bind(addr).unwrap();
        let mut buf = [0u8; rosc::decoder::MTU];
        let handle_packet = |packet: OscPacket| {
            if let OscPacket::Message(msg) = packet {
                if let Some(args) = msg.args {
                    if let OscType::Int(s) = args[0] {
                        match msg.addr.as_ref() {
                            "/steps" => steps.set(s as u8),
                            "/pulses" => pulses.set(s as u8),
                            other => println!("unknown addr {}", other),
                        }
                    }
                }
            }
        };
        loop {
            match sock.recv_from(&mut buf) {
                Ok((size, _addr)) => {
                    let packet = rosc::decoder::decode(&buf[..size]).unwrap();
                    handle_packet(packet);
                }
                Err(e) => {
                    println!("Error receiving from socket: {}", e);
                    break;
                }
            };
        }
    });
    */

    let ntrig = note_trig.clone();
    let trig = GraphNodeWrapper::new_p(FuncWrapper::new_boxed(
        ChildCount::None,
        move |context: &mut dyn SchedContext, _childen: &mut dyn ChildExec| {
            let ntrig = ntrig.lock();
            ntrig.note_with_dur(
                TimeSched::Relative(0),
                TimeResched::Relative(1),
                context.as_schedule_trigger_mut(),
                0,
                0,
                127,
            );
            true
        },
    ));

    let step_gatec = step_gate.clone();
    let gate = GraphNodeWrapper::new_p(FuncWrapper::new_boxed(
        ChildCount::Inf,
        move |context: &mut dyn SchedContext, children: &mut dyn ChildExec| {
            if step_gatec.get() {
                children.exec_all(context);
            }
            children.has_children()
        },
    ));

    let step_seq = NChildGraphNodeWrapper::new_p(StepSeq::new_p(step_ticks, steps));

    let setup = Arc::new(spinlock::Mutex::new(IndexFuncWrapper::new_boxed(
        move |index: usize, _context: &mut dyn SchedContext| {
            if index < gates.len() {
                step_gate.set(gates[index].get());
            }
        },
    )));
    step_seq.lock().index_child_append(LNode::new_boxed(setup));

    let ntrig = note_trig.clone();
    let display = GraphNodeWrapper::new_p(FuncWrapper::new_boxed(
        ChildCount::None,
        move |context: &mut dyn SchedContext, _childen: &mut dyn ChildExec| {
            let ntrig = ntrig.lock();
            let num = (context.context_tick() % 16) as u8 * 2;
            ntrig.note_with_dur(
                TimeSched::Relative(0),
                TimeResched::Relative(4410),
                context.as_schedule_trigger_mut(),
                2,
                num,
                127,
            );
            true
        },
    ));
    gate.lock().child_append(LNode::new_boxed(trig));
    gate.lock().child_append(LNode::new_boxed(display));
    step_seq.lock().child_append(LNode::new_boxed(gate));

    clock.child_append(LNode::new_boxed(step_seq));
    sched.schedule(TimeSched::Relative(0), clock);

    let mut ex = sched.executor().unwrap();
    let process_callback = move |client: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
        //read in midi
        for m in midi_in.iter(ps) {
            if let Some(val) = MidiValue::try_from(m.bytes) {
                if let MidiValue::Note {
                    on: true,
                    chan,
                    num,
                    vel,
                } = val
                {
                    match chan {
                        2 => {
                            let index = (num >> 1) as usize;
                            if index < toggles.len() {
                                let v = !toggles[index].get();
                                toggles[index].set(v);
                                println!("toggle {}, {}", index, v);
                            }
                        }
                        8 => {
                            if let Some(offset) = match num {
                                48 => Some(1.0f32),
                                49 => Some(-1.0f32),
                                _ => None,
                            } {
                                let c = bpm.get() + offset * (1.0 + 5.0 * (vel as f32) / 127f32);
                                bpm.set(c);
                                println!("BPM {}", c);
                            }
                        }
                        _ => (),
                    }
                }
            }
        }

        ex.run(ps.n_frames() as usize, client.sample_rate() as usize);

        //evaluate triggers
        let note_trig = note_trig.lock();
        ex.eval_triggers(&mut |time, index, _block_time, _trig_context| {
            if index == note_trig.trigger_index() {
                note_trig.eval(time);
            }
        });

        //evaluate midi
        let block_time = ex.time_last();
        let mut out_p = midi_out.writer(ps);
        while let Some(midi) = mreceiver.try_recv().ok() {
            let time = (midi.tick() - block_time) as u32 % ps.n_frames();
            let mut iter = midi.value().iter();
            let mut write = |bytes: &[u8]| {
                let _ = out_p.write(&jack::RawMidi { time, bytes });
            };
            match iter.len() {
                3 => write(&[
                    iter.next().unwrap(),
                    iter.next().unwrap(),
                    iter.next().unwrap(),
                ]),
                2 => write(&[iter.next().unwrap(), iter.next().unwrap()]),
                1 => write(&[iter.next().unwrap()]),
                _ => (),
            };
        }

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
