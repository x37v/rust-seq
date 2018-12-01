extern crate jack;
extern crate rosc;

#[macro_use]
extern crate sched;

use sched::binding::bpm;
use sched::binding::generators::*;
use sched::binding::latch::BindingLatch;
use sched::binding::observable::{new_observer_node, Observable, ObservableBinding};
use sched::binding::ops::*;
use sched::binding::spinlock::SpinlockParamBinding;
use sched::binding::{BindingLatchP, BindingP, ParamBinding, ParamBindingGet, ParamBindingSet};

use sched::ptr::{SShrPtr, ShrPtr};

use sched::graph::clock_ratio::ClockRatio;
use sched::graph::gate::Gate;
use sched::graph::index_latch::IndexLatch;
use sched::graph::index_report::IndexReporter;
use sched::graph::midi::MidiNote;
use sched::graph::node_wrapper::{GraphNodeWrapper, NChildGraphNodeWrapper};
use sched::graph::root_clock::RootClock;
use sched::graph::step_seq::StepSeq;
use sched::graph::GraphNode;

use sched::midi::{MidiTrigger, MidiValue};

use sched::spinlock;
use sched::{LNode, Sched, Scheduler, TimeResched, TimeSched};

use std::sync::atomic::{AtomicBool, AtomicUsize};
use std::sync::mpsc::sync_channel;

use std::io;

use sched::quneo_display::DisplayType as QDisplayType;
use sched::quneo_display::{QuNeoDisplay, QuNeoDrawer};

use std::sync::Arc;

struct PageData {
    index: ShrPtr<ObservableBinding<usize, AtomicUsize>>,
    length: ShrPtr<ObservableBinding<usize, AtomicUsize>>,
    gates: Vec<ShrPtr<ObservableBinding<bool, AtomicBool>>>,
    clock_mul: ShrPtr<dyn ParamBinding<u8>>,
    clock_div: ShrPtr<dyn ParamBinding<u8>>,
    probability: ShrPtr<dyn ParamBinding<f32>>,
    volume: ShrPtr<dyn ParamBinding<f32>>,
    volume_rand: ShrPtr<dyn ParamBinding<f32>>,
}

impl PageData {
    pub fn new(
        index: ShrPtr<ObservableBinding<usize, AtomicUsize>>,
        length: ShrPtr<ObservableBinding<usize, AtomicUsize>>,
        gates: Vec<ShrPtr<ObservableBinding<bool, AtomicBool>>>,
        clock_mul: ShrPtr<dyn ParamBinding<u8>>,
        clock_div: ShrPtr<dyn ParamBinding<u8>>,
        probability: ShrPtr<dyn ParamBinding<f32>>,
        volume: ShrPtr<dyn ParamBinding<f32>>,
        volume_rand: ShrPtr<dyn ParamBinding<f32>>,
    ) -> Self {
        Self {
            index,
            length,
            gates,
            clock_mul,
            clock_div,
            probability,
            volume,
            volume_rand,
        }
    }
}

fn main() {
    let (notify_sender, notify_receiver) = sync_channel(16);
    let jack_connections: ShrPtr<ObservableBinding<usize, _>> =
        new_shrptr!(ObservableBinding::new(AtomicUsize::new(0)));
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
    let midi_trig = new_sshrptr!(MidiTrigger::new(msender));

    let bpm_binding = new_sshrptr!(bpm::ClockData::new(120.0, 960));
    let _bpm = new_shrptr!(bpm::ClockBPMBinding(bpm_binding.clone()));
    let _ppq = new_shrptr!(bpm::ClockPPQBinding(bpm_binding.clone()));
    let micros = new_shrptr!(bpm::ClockPeriodMicroBinding(bpm_binding.clone()));
    let mut clock = new_uniqptr!(RootClock::new(micros.clone()));

    let _pulses = SpinlockParamBinding::new_p(2);
    let step_ticks = SpinlockParamBinding::new_p(960 / 4);
    let _step_index = SpinlockParamBinding::new_p(0usize);

    /*
    let addr_s = "127.0.0.1:10001";
    let addr = match SocketAddrV4::from_str(addr_s) {
        Ok(addr) => addr,
        Err(e) => panic!("error with osc address {}", e),
    };
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

    let current_page = new_shrptr!(AtomicUsize::new(0));
    let page_select_shift = new_shrptr!(AtomicBool::new(false));
    let mul_select_shift = new_shrptr!(AtomicBool::new(false));
    let div_select_shift = new_shrptr!(AtomicBool::new(false));
    let len_select_shift = new_shrptr!(AtomicBool::new(false));

    let mut page_data: Vec<SShrPtr<PageData>> = Vec::new();
    let midi_notev_min = new_shrptr!(1u8);
    let midi_max = new_shrptr!(127u8);
    let midi_maxf = new_shrptr!(127f32);

    for page in 0..64 {
        let probability = SpinlockParamBinding::new_p(1f32);

        let volume = SpinlockParamBinding::new_p(1.0f32);
        let volume_rand = SpinlockParamBinding::new_p(0f32);
        let volume_rand_offset = new_shrptr!(GetUniformRand::new(
            new_shrptr!(GetNegate::new(volume_rand.clone())),
            volume_rand.clone(),
        ));

        let velocity = new_shrptr!(GetSum::new(volume.clone(), volume_rand_offset));
        let velocity = new_shrptr!(GetMul::new(velocity.clone(), midi_maxf.clone()));
        let velocity: ShrPtr<GetCast<f32, u8, _>> = new_shrptr!(GetCast::new(velocity));
        let velocity = new_shrptr!(GetClamp::new(
            velocity,
            midi_notev_min.clone(),
            midi_max.clone(),
        ));

        let steps = new_shrptr!(ObservableBinding::new(AtomicUsize::new(16)));
        let note = new_shrptr!((page + 36) as u8);

        //build up gates
        let gates: Vec<ShrPtr<ObservableBinding<bool, _>>> = vec![false; 64]
            .iter()
            .map(|v| new_shrptr!(ObservableBinding::new(AtomicBool::new(*v))))
            .collect();
        for g in gates.iter() {
            g.add_observer(new_observer_node(notify_sender.clone()));
        }

        let step_gate = new_shrptr!(AtomicBool::new(false));
        let latches: Vec<BindingLatchP> = gates
            .iter()
            .map(|g| new_shrptr!(BindingLatch::new(g.clone(), step_gate.clone())) as BindingLatchP)
            .collect();

        let trig = GraphNodeWrapper::new_p(MidiNote::new_p(
            midi_trig.clone(),
            new_shrptr!(9u8),
            note.clone(),
            new_shrptr!(TimeResched::Relative(1)),
            velocity.clone(),
            new_shrptr!(127u8),
        ));

        let gate = GraphNodeWrapper::new_p(Gate::new_p(step_gate.clone()));
        let step_seq =
            NChildGraphNodeWrapper::new_p(StepSeq::new_p(step_ticks.clone(), steps.clone()));

        let index_binding = new_shrptr!(ObservableBinding::new(AtomicUsize::new(0)));
        let index_latch = IndexLatch::new_p(latches);
        step_seq
            .lock()
            .index_child_append(LNode::new_boxed(index_latch));

        let index_report = IndexReporter::new_p(index_binding.clone());
        step_seq
            .lock()
            .index_child_append(LNode::new_boxed(index_report));

        let uniform = new_shrptr!(GetUniformRand::new(new_shrptr!(0f32), new_shrptr!(1f32)));
        let cmp = new_shrptr!(GetCmp::new(CmpOp::Greater, probability.clone(), uniform));
        let prob = GraphNodeWrapper::new_p(Gate::new_p(cmp.clone()));
        prob.lock().child_append(LNode::new_boxed(trig));

        gate.lock().child_append(LNode::new_boxed(prob));

        step_seq.lock().child_append(LNode::new_boxed(gate));

        let mul = SpinlockParamBinding::new_p(1);
        let div = SpinlockParamBinding::new_p(1);

        page_data.push(new_sshrptr!(PageData::new(
            index_binding.clone(),
            steps.clone(),
            gates.iter().cloned().collect(),
            mul.clone(),
            div.clone(),
            probability.clone(),
            volume.clone(),
            volume_rand.clone(),
        )));

        index_binding.add_observer(new_observer_node(notify_sender.clone()));

        let ratio = GraphNodeWrapper::new_p(ClockRatio::new_p(mul, div));
        ratio.lock().child_append(LNode::new_boxed(step_seq));
        clock.child_append(LNode::new_boxed(ratio));
    }

    sched.schedule(TimeSched::Relative(0), clock);

    let cpage = current_page.clone();
    let draw_data: Vec<SShrPtr<PageData>> = page_data.iter().cloned().collect();
    jack_connections.add_observer(new_observer_node(notify_sender));
    let force_id = jack_connections.id();

    let mul_select_shiftc = mul_select_shift.clone();
    let div_select_shiftc = div_select_shift.clone();
    let len_select_shiftc = len_select_shift.clone();
    let page_select_shiftc = page_select_shift.clone();

    let draw_one = |display: &mut QuNeoDisplay, index: usize, value: u8| {
        for i in 0..64 {
            display.update(QDisplayType::Pad, i, 0u8);
        }
        display.update(QDisplayType::Pad, index, value);
    };

    let drawer = new_uniqptr!(QuNeoDrawer::new(
        midi_trig.clone(),
        TimeResched::Relative(441),
        new_uniqptr!(move |display: &mut QuNeoDisplay| {
            //TODO make sure the notification is actually something we care about
            let force = notify_receiver.try_iter().any(|x| x == force_id);
            let page = cpage.get();
            if page_select_shiftc.get() {
                draw_one(display, page, 127u8);
            } else {
                if page < draw_data.len() {
                    let page = draw_data[page].lock();
                    if len_select_shiftc.get() {
                        draw_one(display, page.length.get() - 1, 64u8);
                    } else if div_select_shiftc.get() {
                        draw_one(display, page.clock_div.get() as usize - 1, 127u8);
                    } else if mul_select_shiftc.get() {
                        draw_one(display, page.clock_mul.get() as usize - 1, 127u8);
                    } else {
                        for i in 0..page.gates.len() {
                            display.update(
                                QDisplayType::Pad,
                                i,
                                if page.gates[i].get() { 127u8 } else { 0u8 },
                            );
                        }
                        let index = page.index.get();
                        if index < 64 {
                            display.update(QDisplayType::Pad, index, 32);
                        }
                        display.update(QDisplayType::Slider, 4, (127f32 * page.volume.get()) as u8);
                        display.update(
                            QDisplayType::Slider,
                            5,
                            (127f32 * page.volume_rand.get()) as u8,
                        );
                        display.update(
                            QDisplayType::Slider,
                            7,
                            (127f32 * page.probability.get()) as u8,
                        );
                    }
                }
                if force {
                    display.force_draw();
                }
            }
        }),
    ));

    sched.schedule(TimeSched::Relative(0), drawer);

    let mut ex = sched.executor().unwrap();
    ex.add_trigger(LNode::new_boxed(midi_trig.clone()));
    let process_callback = move |client: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
        //read in midi
        for m in midi_in.iter(ps) {
            if let Some(val) = MidiValue::try_from(m.bytes) {
                match val {
                    MidiValue::Note {
                        on,
                        chan,
                        num,
                        vel: _,
                    } => {
                        match chan {
                            15 => {
                                let page = current_page.get();
                                if page < page_data.len() {
                                    let page = page_data[page].lock();
                                    let index = num as usize;
                                    if index < page.gates.len() {
                                        if on {
                                            if page_select_shift.get() {
                                                current_page.set(index);
                                            } else if len_select_shift.get() {
                                                page.length.set(index + 1);
                                            } else if div_select_shift.get() {
                                                page.clock_div.set((index + 1) as u8);
                                            } else if mul_select_shift.get() {
                                                page.clock_mul.set((index + 1) as u8);
                                            } else {
                                                let v = !page.gates[index].get();
                                                page.gates[index].set(v);
                                            }
                                        }
                                    } else {
                                        match index {
                                            67 => page_select_shift.set(on),
                                            76 => mul_select_shift.set(on),
                                            77 => div_select_shift.set(on),
                                            79 => len_select_shift.set(on),
                                            _ => (),
                                        }
                                    }
                                }
                            }
                            /*
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
                            */
                            _ => (),
                        }
                    }
                    MidiValue::ContCtrl { chan: 15, num, val } => {
                        let page = current_page.get();
                        if page < page_data.len() {
                            let page = page_data[page].lock();
                            match num {
                                102 => page.volume.set(val as f32 / 127f32),
                                103 => page.volume_rand.set(val as f32 / 127f32),
                                105 => page.probability.set(val as f32 / 127f32),
                                _ => (),
                            }
                        }
                    }
                    _ => (),
                }
            }
        }

        ex.run(ps.n_frames() as usize, client.sample_rate() as usize);

        let mut out_p = midi_out.writer(ps);
        let mut write_midi = |time: u32, bytes: &[u8]| {
            let _ = out_p.write(&jack::RawMidi { time, bytes });
        };
        let mut write_midi_value = |time: u32, value: &MidiValue| {
            let mut iter = value.iter();
            match iter.len() {
                3 => write_midi(
                    time,
                    &[
                        iter.next().unwrap(),
                        iter.next().unwrap(),
                        iter.next().unwrap(),
                    ],
                ),
                2 => write_midi(time, &[iter.next().unwrap(), iter.next().unwrap()]),
                1 => write_midi(time, &[iter.next().unwrap()]),
                _ => (),
            };
        };

        //evaluate midi
        let block_time = ex.time_last();
        while let Some(midi) = mreceiver.try_recv().ok() {
            let time = (midi.tick() - block_time) as u32 % ps.n_frames();
            write_midi_value(time, midi.value());
        }

        jack::Control::Continue
    };

    let process = jack::ClosureProcessHandler::new(process_callback);

    let notify = Notifications::new(jack_connections);

    // Activate the client, which starts the processing.
    let active_client = client.activate_async(notify, process).unwrap();

    // Wait for user input to quit
    println!("Press enter/return to quit...");
    let mut user_input = String::new();
    io::stdin().read_line(&mut user_input).ok();

    active_client.deactivate().unwrap();
}

struct Notifications {
    connection_count: BindingP<usize>,
}

impl Notifications {
    pub fn new(connection_count: BindingP<usize>) -> Self {
        Self { connection_count }
    }
}

impl jack::NotificationHandler for Notifications {
    fn thread_init(&self, _: &jack::Client) {
        //println!("JACK: thread init");
    }

    fn shutdown(&mut self, _status: jack::ClientStatus, _reason: &str) {
        /*
        println!(
            "JACK: shutdown with status {:?} because \"{}\"",
            status, reason
        );
        */
    }

    fn freewheel(&mut self, _: &jack::Client, _is_enabled: bool) {
        /*
        println!(
            "JACK: freewheel mode is {}",
            if is_enabled { "on" } else { "off" }
        );
        */
    }

    fn buffer_size(&mut self, _: &jack::Client, _sz: jack::Frames) -> jack::Control {
        //println!("JACK: buffer size changed to {}", sz);
        jack::Control::Continue
    }

    fn sample_rate(&mut self, _: &jack::Client, _srate: jack::Frames) -> jack::Control {
        //println!("JACK: sample rate changed to {}", srate);
        jack::Control::Continue
    }

    fn client_registration(&mut self, _: &jack::Client, _name: &str, _is_reg: bool) {
        /*
        println!(
            "JACK: {} client with name \"{}\"",
            if is_reg { "registered" } else { "unregistered" },
            name
        );
        */
    }

    fn port_registration(&mut self, _: &jack::Client, _port_id: jack::PortId, _is_reg: bool) {
        /*
        println!(
            "JACK: {} port with id {}",
            if is_reg { "registered" } else { "unregistered" },
            port_id
        );
        */
    }

    fn port_rename(
        &mut self,
        _: &jack::Client,
        _port_id: jack::PortId,
        _old_name: &str,
        _new_name: &str,
    ) -> jack::Control {
        /*
        println!(
            "JACK: port with id {} renamed from {} to {}",
            port_id, old_name, new_name
        );
        */
        jack::Control::Continue
    }

    fn ports_connected(
        &mut self,
        _: &jack::Client,
        _port_id_a: jack::PortId,
        _port_id_b: jack::PortId,
        are_connected: bool,
    ) {
        let c = self.connection_count.get();
        if are_connected {
            self.connection_count.set(1 + c);
        } else if c > 0 {
            self.connection_count.set(c - 1);
        }
        /*
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
        */
    }

    fn graph_reorder(&mut self, _: &jack::Client) -> jack::Control {
        //println!("JACK: graph reordered");
        jack::Control::Continue
    }

    fn xrun(&mut self, _: &jack::Client) -> jack::Control {
        //println!("JACK: xrun occurred");
        jack::Control::Continue
    }

    fn latency(&mut self, _: &jack::Client, _mode: jack::LatencyType) {
        /*
        println!(
            "JACK: {} latency has changed",
            match mode {
                jack::LatencyType::Capture => "capture",
                jack::LatencyType::Playback => "playback",
            }
        );
        */
    }
}
