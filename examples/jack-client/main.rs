use jack;
use std::io;

extern crate alloc;
mod page;
mod quneo_display;

use quneo_display::{DisplayType as QDisplayType, QuNeoDisplay, QuNeoDrawer};

use core::convert::Into;

use alloc::sync::Arc;
use spin::Mutex;

use sched::{
    binding::*,
    event::bindstore::BindStoreEvent,
    event::*,
    graph::{
        bindstore::BindStoreNode, children, clock_ratio::ClockRatio, fanout::FanOut,
        midi::MidiNote, node_wrapper::GraphNodeWrapper, root_clock::RootClock,
        root_wrapper::GraphRootWrapper, step_seq::StepSeq, GraphNodeContainer,
    },
    item_sink::{ItemDispose, ItemSink},
    item_source::*,
    midi::MidiValue,
    pqueue::*,
    schedule::ScheduleExecutor,
    tick::*,
};

use core::ops::Mul;

use core::sync::atomic::{AtomicBool, AtomicUsize};

type MidiEnqueue = Arc<spin::Mutex<dyn TickPriorityEnqueue<MidiValue>>>;
type TickedMidiValueEvent = sched::graph::midi::TickedMidiValueEvent<MidiEnqueue>;
type BindStoreEventBool = BindStoreEvent<bool, bool, Arc<Mutex<dyn ParamBindingSet<bool>>>>;

static JACK_CONNECTION_COUNT: AtomicUsize = AtomicUsize::new(0);

pub trait IntoPtrs {
    fn into_arc(self) -> Arc<Self>;
    fn into_alock(self) -> Arc<Mutex<Self>>;
}

impl<T> IntoPtrs for T
where
    T: Sized,
{
    fn into_arc(self) -> Arc<Self> {
        Arc::new(self)
    }
    fn into_alock(self) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(self))
    }
}

fn main() {
    let midi_queue: Arc<spin::Mutex<BinaryHeapQueue<MidiValue>>> = Default::default();
    let sched_queue: Arc<spin::Mutex<BinaryHeapQueue<EventContainer>>> = Default::default();
    let (client, _status) =
        jack::Client::new("xnor_sched", jack::ClientOptions::NO_START_SERVER).unwrap();

    let mut midi_out = client
        .register_port("midi", jack::MidiOut::default())
        .unwrap();

    let midi_in = client
        .register_port("control", jack::MidiIn::default())
        .unwrap();

    let mut page_data: Vec<Arc<spin::Mutex<page::PageData>>> = Vec::new();
    let current_page: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));

    let (dispose_sink, dispose) = sched::std::channel_item_sink::channel_item_sink(1024);
    let dispose_sink: Arc<Mutex<dyn ItemSink<EventContainer>>> = dispose_sink.into_alock();

    let mut ex = ScheduleExecutor::new(
        dispose_sink,
        sched_queue.clone() as Arc<spin::Mutex<dyn TickPriorityDequeue<EventContainer>>>,
        sched_queue.clone() as Arc<spin::Mutex<dyn TickPriorityEnqueue<EventContainer>>>,
    );

    let ppq = 960usize;
    let step_ticks = AtomicUsize::new(ppq / 4usize).into_arc();

    let mul_select_shift = AtomicBool::new(false).into_arc();
    let div_select_shift = AtomicBool::new(false).into_arc();
    let len_select_shift = AtomicBool::new(false).into_arc();

    let clock_binding: Arc<Mutex<dyn bpm::Clock>> = bpm::ClockData::new(120.0, ppq).into_alock();
    let bpm = bpm::ClockBPMBinding(clock_binding.clone()).into_arc();
    let _ppq = bpm::ClockPPQBinding(clock_binding.clone()).into_arc();
    let micros: Arc<dyn ParamBindingGet<f64>> =
        bpm::ClockPeriodMicroBinding(clock_binding.clone()).into_arc();

    let (mut midi_creator, midi_source) = sched::std::channel_item_source::item_source(1024);
    let midi_source: Arc<Mutex<dyn ItemSource<TickedMidiValueEvent>>> =
        Arc::new(Mutex::new(midi_source));

    let (mut boolbind_creator, boolbind_source) =
        sched::std::channel_item_source::item_source(1024);
    let boolbind_source: Arc<Mutex<dyn ItemSource<BindStoreEventBool>>> =
        Arc::new(Mutex::new(boolbind_source));

    let retrig_stages = 4f64;
    let mut voices = Vec::new();
    let mut trigon_oneshots = Vec::new();
    let mut trigoff_oneshots = Vec::new();
    for page_index in 0..32 {
        let data = page::PageData::new();
        let note = page_index;

        //root -> ratio --+-> step_seq -(nchild index bind) [selectively open gate etc]
        //                +-> step_gate-> ratio (retrig)-+-> note
        //                                               +-> notify

        let ratio = ClockRatio::new(
            data.clock_mul.clone() as Arc<dyn ParamBindingGet<_>>,
            data.clock_div.clone() as Arc<dyn ParamBindingGet<_>>,
        );

        let seq = StepSeq::new(
            step_ticks.clone() as Arc<dyn ParamBindingGet<_>>,
            data.length.clone() as Arc<dyn ParamBindingGet<_>>,
        );

        let gates: Vec<Arc<dyn ParamBindingGet<bool>>> = data
            .gates
            .iter()
            .map(|g| g.clone() as Arc<dyn ParamBindingGet<bool>>)
            .collect();
        let step_gate = ops::GetIndexed::new(
            gates,
            data.step_cur.clone() as Arc<dyn ParamBindingGet<usize>>,
        )
        .into_alock();

        let step_gate =
            sched::graph::gate::Gate::new(step_gate as Arc<Mutex<dyn ParamBindingGet<bool>>>);
        let onvel = ops::GetUnaryOp::new(
            ops::funcs::cast_or_default,
            ops::GetBinaryOp::new(
                f64::mul,
                127f64,
                data.volume.clone() as Arc<dyn ParamBindingGet<f64>>,
            )
            .into_alock() as Arc<Mutex<dyn ParamBindingGet<f64>>>,
        )
        .into_alock() as Arc<Mutex<dyn ParamBindingGet<u8>>>;

        let note = MidiNote::new(
            &0,
            note,
            &TickResched::ContextRelative(1),
            onvel,
            &127,
            midi_source.clone(),
            midi_queue.clone() as MidiEnqueue,
        );

        //one shot bound node lets us know if this node has been triggered since we last read
        //from the one shot
        let os_on = generators::GetOneShot::new().into_alock();
        let os_off = generators::GetOneShot::new().into_alock();
        let trig = BindStoreNode::new(true, os_on.clone() as Arc<Mutex<dyn ParamBindingSet<bool>>>);
        let trig: GraphNodeContainer =
            GraphNodeWrapper::new(trig, children::empty::Children).into();

        trigon_oneshots.push(os_on as Arc<Mutex<dyn ParamBindingGet<bool>>>);
        trigoff_oneshots.push(os_off);

        let note: GraphNodeContainer =
            GraphNodeWrapper::new(note, children::empty::Children).into();

        //note + trigger indication: fanout
        let note: GraphNodeContainer = GraphNodeWrapper::new(
            FanOut::new(),
            children::boxed::Children::new(Box::new([note, trig])),
        )
        .into();

        //the sequencer doesn't actually children, it just sets an index here
        //child is () aka noop
        let seq: GraphNodeContainer = GraphNodeWrapper::new(
            seq,
            children::nchild::ChildWrapper::new_with_index(
                (),
                data.step_cur.clone() as Arc<dyn ParamBindingSet<usize>>,
            ),
        )
        .into();

        let exp = data.retrig_amount.clone() as Arc<dyn ParamBindingGet<f64>>;

        let exp = ops::GetUnaryOp::new(ops::funcs::cast_or_default, exp).into_alock()
            as Arc<Mutex<dyn ParamBindingGet<usize>>>;

        let div = ops::GetBinaryOp::new(num::pow::pow, 2usize, exp).into_alock()
            as Arc<Mutex<dyn ParamBindingGet<usize>>>;
        let retrig_div = ops::GetBinaryOp::new(ops::funcs::div_protected, ppq / 4, div).into_alock()
            as Arc<Mutex<dyn ParamBindingGet<usize>>>;

        let retrig_ratio: GraphNodeContainer = GraphNodeWrapper::new(
            ClockRatio::new(1usize, retrig_div as Arc<Mutex<dyn ParamBindingGet<usize>>>),
            children::boxed::Children::new(Box::new([note.clone()])),
        )
        .into();

        let step_gate: GraphNodeContainer = GraphNodeWrapper::new(
            step_gate,
            children::boxed::Children::new(Box::new([retrig_ratio])),
        )
        .into();

        let ratio: GraphNodeContainer = GraphNodeWrapper::new(
            ratio,
            children::boxed::Children::new(Box::new([seq, step_gate])),
        )
        .into();

        page_data.push(Arc::new(Mutex::new(data)));

        voices.push(ratio);
    }

    let fanout: GraphNodeContainer = GraphNodeWrapper::new(
        FanOut::new(),
        children::boxed::Children::new(voices.into_boxed_slice()),
    )
    .into();

    let root = EventContainer::new(GraphRootWrapper::new(
        RootClock::new(micros),
        children::boxed::Children::new(Box::new([fanout])),
    ));
    assert!(sched_queue.lock().enqueue(0, root).is_ok());

    //draw
    {
        let cpage = current_page.clone();
        let draw_data: Vec<_> = page_data.to_vec();
        let mul_select_shiftc = mul_select_shift.clone();
        let div_select_shiftc = div_select_shift.clone();
        let len_select_shiftc = len_select_shift.clone();

        let draw_one =
            |display: &mut QuNeoDisplay, index: usize, value: u8, start: usize, end: usize| {
                for i in start..end {
                    display.update(QDisplayType::Pad, i, 0u8);
                }
                display.update(QDisplayType::Pad, index, value);
            };

        let connections = Arc::new(AtomicUsize::new(0));
        let page_last = Arc::new(AtomicUsize::new(0));
        let draw = Box::new(
            move |display: &mut QuNeoDisplay, context: &mut dyn EventEvalContext| {
                let page = cpage.get();
                //display.force_draw();
                let pages = draw_data.len();

                //turn off the old page display if the page has changed
                let last = page_last.get();
                if page != last {
                    display.update(QDisplayType::Pad, last, 0);
                    page_last.set(page);
                }

                for p in 0..pages {
                    //indicate the current page
                    //flash page buttons for off page sequences when they are triggered
                    if p == page {
                        display.update(QDisplayType::Pad, p, 127u8);
                    } else {
                        //got a trigger, turn pad on
                        //then schedule off
                        let os_on = &trigon_oneshots[p];
                        let os_off = &trigoff_oneshots[p];
                        if os_on.get() {
                            display.update(QDisplayType::Pad, p, 64u8);
                            let off = boolbind_source.lock().try_get(BindStoreEvent::new(
                                true,
                                os_off.clone() as Arc<Mutex<dyn ParamBindingSet<bool>>>,
                            ));
                            if let Ok(off) = off {
                                let r = context.event_schedule(
                                    TickSched::Relative(4410),
                                    EventContainer::new_from_box(off),
                                );
                                if r.is_err() {
                                    println!("failed to schedule off event");
                                }
                            } else {
                                println!("failed to get off event");
                            }
                        } else if os_off.lock().get() {
                            display.update(QDisplayType::Pad, p, 0);
                        }
                    }
                }

                if page < pages {
                    let offset = pages;
                    let page = draw_data[page].lock();
                    if len_select_shiftc.get() {
                        draw_one(display, offset + page.length.get() - 1, 64u8, offset, 64);
                    } else if div_select_shiftc.get() {
                        draw_one(
                            display,
                            offset + page.clock_div.get() as usize - 1,
                            127u8,
                            offset,
                            64,
                        );
                    } else if mul_select_shiftc.get() {
                        draw_one(
                            display,
                            offset + page.clock_mul.get() as usize - 1,
                            127u8,
                            offset,
                            64,
                        );
                    } else {
                        //display the state of the gates
                        let step = page.step_cur.get();
                        for i in 0..page.gates.len() {
                            let v = if page.gates[i].get() {
                                if i == step {
                                    127u8 //on and current
                                } else {
                                    64u8 //on not current
                                }
                            } else if i == step {
                                32u8 // off current
                            } else {
                                0u8 // off
                            };
                            display.update(QDisplayType::Pad, offset + i, v);
                        }
                        display.update(QDisplayType::Slider, 4, (127f64 * page.volume.get()) as u8);
                        display.update(
                            QDisplayType::Slider,
                            5,
                            (127f64 * page.volume_rand.get()) as u8,
                        );
                        display.update(
                            QDisplayType::Slider,
                            7,
                            (127f64 * page.probability.get()) as u8,
                        );
                    }
                }

                //force a redraw if connection count changes
                {
                    let last = connections.get();
                    let cur = JACK_CONNECTION_COUNT.get();
                    if last != cur {
                        connections.set(cur);
                        display.force_draw();
                    }
                }
            },
        );
        let draw = QuNeoDrawer::new(
            midi_queue.clone() as MidiEnqueue,
            TickResched::Relative(441),
            draw,
        );
        let draw = EventContainer::new(draw);
        assert!(sched_queue.lock().enqueue(0, draw).is_ok());
    }

    let mut fill_dispose = move || {
        midi_creator.fill().expect("failed to fill midi");
        boolbind_creator.fill().expect("failed to fill boolbind");
        dispose.dispose_all().expect("dispose failed");
    };

    println!("starting dispose thread");
    fill_dispose();
    std::thread::spawn(move || loop {
        fill_dispose();
        std::thread::sleep(std::time::Duration::from_millis(1));
    });

    let update_bpm = move |offset: f64, vel: u8| {
        let c = bpm.get() + offset * (1.0 + 5.0 * (vel as f64) / 127f64);
        bpm.set(c);
    };

    let retrig_update = Arc::new(spinlock::SpinlockParamBinding::new(0f64));
    let retrig_hysteresis = Arc::new(Mutex::new(hysteresis::Hysteresis::new(
        retrig_update.clone() as Arc<dyn ParamBindingGet<f64>>,
        0.1f64,
    ))) as Arc<Mutex<dyn ParamBindingGet<f64>>>;
    let process_callback = move |client: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
        let process_note = |on: bool, chan: u8, num: u8, vel: u8| {
            if chan == 15 {
                let pages = page_data.len();
                let page = current_page.get();
                if page < pages {
                    let page = page_data[page].lock();
                    let mut index = num as usize;
                    if index < pages {
                        if on {
                            current_page.set(index);
                        }
                    } else if index - pages < page.gates.len() {
                        index -= pages;
                        if on {
                            if len_select_shift.get() {
                                page.length.set(index + 1);
                            } else if div_select_shift.get() {
                                page.clock_div.set(index + 1);
                            } else if mul_select_shift.get() {
                                page.clock_mul.set(index + 1);
                            } else {
                                let v = !page.gates[index].get();
                                page.gates[index].set(v);
                            }
                        }
                    } else {
                        match index {
                            90 => {
                                //turn on in the parameter setting
                                if !on {
                                    page.retrig.set(false);
                                }
                            }
                            67 => len_select_shift.set(on),
                            76 => mul_select_shift.set(on),
                            77 => div_select_shift.set(on),
                            78 => {
                                if on {
                                    update_bpm(1f64, vel);
                                }
                            }
                            79 => {
                                if on {
                                    update_bpm(-1f64, vel);
                                }
                            }
                            _ => (),
                        }
                    }
                }
            }
        };

        //read in midi
        for m in midi_in.iter(ps) {
            if let Some(val) = MidiValue::try_from(m.bytes) {
                match val {
                    MidiValue::NoteOn { chan, num, vel } => process_note(true, chan, num, vel),
                    MidiValue::NoteOff { chan, num, vel } => process_note(false, chan, num, vel),
                    MidiValue::ContCtrl { chan: 15, num, val } => {
                        let page = current_page.get();
                        if page < page_data.len() {
                            let page = page_data[page].lock();
                            match num {
                                102 => page.volume.set(val as f64 / 127f64),
                                103 => page.volume_rand.set(val as f64 / 127f64),
                                105 => page.probability.set(val as f64 / 127f64),
                                106 => {
                                    retrig_update.set(retrig_stages * (val as f64 / 127f64));
                                    page.retrig_amount.set(retrig_hysteresis.get());
                                    page.retrig.set(true); //wait for a value to set this
                                }
                                _ => (),
                            }
                        }
                    }
                    _ => (),
                }
            }
        }

        let now = ex.tick_next();
        ex.run(ps.n_frames() as usize, client.sample_rate() as usize);

        {
            let mut out_p = midi_out.writer(ps);
            let mut write_midi = |tick: u32, bytes: &[u8]| {
                let _ = out_p.write(&jack::RawMidi { time: tick, bytes });
            };
            let mut q = midi_queue.lock();
            let next = ex.tick_next();
            while let Some((t, midi)) = q.dequeue_lt(next) {
                let tick = (if t < now { now } else { t } - now) as u32;
                let iter = &mut midi.iter();
                match iter.len() {
                    3 => write_midi(
                        tick,
                        &[
                            iter.next().unwrap(),
                            iter.next().unwrap(),
                            iter.next().unwrap(),
                        ],
                    ),
                    2 => write_midi(tick, &[iter.next().unwrap(), iter.next().unwrap()]),
                    1 => write_midi(tick, &[iter.next().unwrap()]),
                    _ => (),
                };
            }
        }

        jack::Control::Continue
    };

    let process = jack::ClosureProcessHandler::new(process_callback);

    let notify = Notifications::new();

    // Activate the client, which starts the processing.
    let active_client = client.activate_async(notify, process).unwrap();

    // Wait for user input to quit
    println!("Press enter/return to quit...");
    let mut user_input = String::new();
    io::stdin().read_line(&mut user_input).ok();

    active_client.deactivate().unwrap();
}

struct Notifications {}

impl Notifications {
    pub fn new() -> Self {
        Notifications {}
    }
}

impl jack::NotificationHandler for Notifications {
    fn ports_connected(
        &mut self,
        _: &jack::Client,
        _port_id_a: jack::PortId,
        _port_id_b: jack::PortId,
        are_connected: bool,
    ) {
        let c = JACK_CONNECTION_COUNT.get();
        if are_connected {
            JACK_CONNECTION_COUNT.set(c + 1);
        } else if c > 0 {
            JACK_CONNECTION_COUNT.set(c - 1);
        }
    }
}
