use jack;
use std::io;

extern crate alloc;
mod page;
mod quneo_display;

use quneo_display::{DisplayType as QDisplayType, QuNeoDisplay, QuNeoDrawer};

use core::convert::Into;

use sched::tick::*;

//use sched::event::ticked_value_queue::TickedValueQueueEvent;
use sched::event::*;
use sched::item_sink::ItemSink;
use sched::item_source::ItemSource;
use sched::midi::*;
use sched::pqueue::*;
use sched::schedule::ScheduleExecutor;

use alloc::sync::Arc;
use spin::Mutex;

use sched::graph::*;
use sched::graph::{
    bindstore::BindStoreIndexChild, clock_ratio::ClockRatio, fanout::FanOut,
    node_wrapper::GraphNodeWrapper, root_clock::RootClock, step_seq::StepSeq,
};

use sched::binding::*;

use core::mem::{self, MaybeUninit};

use heapless::binary_heap::{BinaryHeap, Min};
use heapless::consts::*;
use heapless::mpmc::Q64;

use core::sync::atomic::{AtomicBool, AtomicUsize};

pub struct ScheduleQueue(BinaryHeap<TickItem<EventContainer>, U1024, Min>);
pub struct MidiQueue(BinaryHeap<TickItem<MidiValue>, U1024, Min>);
pub struct DisposeSink(Q64<EventContainer>);
pub struct MidiItemSource(Q64<Box<MaybeUninit<TickedMidiValueEvent>>>);

type MidiEnqueue = &'static spin::Mutex<dyn TickPriorityEnqueue<MidiValue>>;
type TickedMidiValueEvent = midi::TickedMidiValueEvent<MidiEnqueue>;

impl TickPriorityEnqueue<EventContainer> for ScheduleQueue {
    fn enqueue(&mut self, tick: usize, value: EventContainer) -> Result<(), EventContainer> {
        let item: TickItem<EventContainer> = (tick, value).into();
        match self.0.push(item) {
            Ok(()) => Ok(()),
            Err(item) => {
                let (_, item) = item.into();
                Err(item)
            }
        }
    }
}
impl TickPriorityDequeue<EventContainer> for ScheduleQueue {
    fn dequeue_lt(&mut self, tick: usize) -> Option<(usize, EventContainer)> {
        if let Some(h) = self.0.peek() {
            if h.tick() < tick {
                //unchecked because we've already peeked
                Some(unsafe { self.0.pop_unchecked().into() })
            } else {
                None
            }
        } else {
            None
        }
    }
}
impl TickPriorityEnqueue<MidiValue> for MidiQueue {
    fn enqueue(&mut self, tick: usize, value: MidiValue) -> Result<(), MidiValue> {
        let item: TickItem<MidiValue> = (tick, value).into();
        match self.0.push(item) {
            Ok(()) => Ok(()),
            Err(item) => {
                let (_, item) = item.into();
                Err(item)
            }
        }
    }
}
impl TickPriorityDequeue<MidiValue> for MidiQueue {
    fn dequeue_lt(&mut self, tick: usize) -> Option<(usize, MidiValue)> {
        if let Some(h) = self.0.peek() {
            if h.tick() < tick {
                //unchecked because we've already peeked
                Some(unsafe { self.0.pop_unchecked().into() })
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl ItemSink<EventContainer> for &'static DisposeSink {
    fn try_put(&mut self, item: EventContainer) -> Result<(), EventContainer> {
        self.0.enqueue(item)
    }
}

impl DisposeSink {
    pub fn dequeue(&self) -> Option<EventContainer> {
        self.0.dequeue()
    }
}

impl ItemSource<TickedMidiValueEvent, Box<TickedMidiValueEvent>> for &'static MidiItemSource {
    fn try_get(
        &mut self,
        init: TickedMidiValueEvent,
    ) -> Result<Box<TickedMidiValueEvent>, TickedMidiValueEvent> {
        if let Some(mut item) = self.0.dequeue() {
            unsafe {
                item.as_mut_ptr().write(init);
                Ok(mem::transmute(item))
            }
        } else {
            println!("failed to get from MidiItemSource");
            Err(init)
        }
    }
}

impl MidiItemSource {
    pub fn fill(&self) {
        while let Ok(()) = self.0.enqueue(Box::new(MaybeUninit::uninit())) {}
    }
}

/*
struct GraphPrinter;

impl GraphLeafExec for GraphPrinter {
    fn graph_exec(&mut self, context: &mut dyn EventEvalContext) {
        println!(
            "graph_exec {} {}",
            context.tick_now(),
            context.context_tick_now()
        );
    }
}
*/

static DISPOSE_SINK: DisposeSink = DisposeSink(Q64::new());
static SCHEDULE_QUEUE: spin::Mutex<ScheduleQueue> =
    spin::Mutex::new(ScheduleQueue(BinaryHeap(heapless::i::BinaryHeap::new())));
static MIDI_QUEUE: spin::Mutex<MidiQueue> =
    spin::Mutex::new(MidiQueue(BinaryHeap(heapless::i::BinaryHeap::new())));

static MIDI_VALUE_SOURCE: MidiItemSource = MidiItemSource(Q64::new());
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

    let mut ex = ScheduleExecutor::new(
        &DISPOSE_SINK,
        &SCHEDULE_QUEUE as &'static spin::Mutex<dyn TickPriorityDequeue<EventContainer>>,
        &SCHEDULE_QUEUE as &'static spin::Mutex<dyn TickPriorityEnqueue<EventContainer>>,
    );

    let ppq = 980usize;
    let step_ticks = AtomicUsize::new(ppq / 4usize).into_arc();

    let mul_select_shift = AtomicBool::new(false).into_arc();
    let div_select_shift = AtomicBool::new(false).into_arc();
    let len_select_shift = AtomicBool::new(false).into_arc();

    let clock_binding: Arc<Mutex<dyn bpm::Clock>> = bpm::ClockData::new(120.0, ppq).into_alock();
    let bpm = bpm::ClockBPMBinding(clock_binding.clone()).into_arc();
    let _ppq = bpm::ClockPPQBinding(clock_binding.clone()).into_arc();
    let micros: Arc<dyn ParamBindingGet<f32>> =
        bpm::ClockPeriodMicroBinding(clock_binding.clone()).into_arc();

    let mut voices = Vec::new();
    for page_index in 0..32 {
        let data = page::PageData::new();
        let note = page_index;

        //root -> ratio -> step_seq ---(nchild index bind)--> step_gate -> note

        let ratio = ClockRatio::new(
            data.clock_mul.clone() as Arc<dyn ParamBindingGet<_>>,
            data.clock_div.clone() as Arc<dyn ParamBindingGet<_>>,
        );

        let seq = StepSeq::new(
            step_ticks.clone() as Arc<dyn ParamBindingGet<_>>,
            data.length.clone() as Arc<dyn ParamBindingGet<_>>,
        );

        let step_cur_bind = IndexChildContainer::new(BindStoreIndexChild::new(
            data.step_cur.clone() as Arc<dyn ParamBindingSet<usize>>,
        ));

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

        let step_gate = gate::Gate::new(step_gate as Arc<Mutex<dyn ParamBindingGet<bool>>>);

        let note = midi::MidiNote::new(
            &0,
            note,
            &TickResched::ContextRelative(1),
            &127,
            &127,
            &MIDI_VALUE_SOURCE,
            &MIDI_QUEUE as MidiEnqueue,
        );

        let note: GraphNodeContainer =
            GraphNodeWrapper::new(note, children::empty::Children).into();

        let step_gate: GraphNodeContainer =
            GraphNodeWrapper::new(step_gate, children::boxed::Children::new(Box::new([note])))
                .into();

        let ichild = children::boxed::IndexChildren::new(Box::new([step_cur_bind]));

        let seq: GraphNodeContainer =
            GraphNodeWrapper::new(seq, children::nchild::ChildWrapper::new(step_gate, ichild))
                .into();

        let ratio: GraphNodeContainer =
            GraphNodeWrapper::new(ratio, children::boxed::Children::new(Box::new([seq]))).into();

        page_data.push(Arc::new(Mutex::new(data)));
        voices.push(ratio);
    }

    let fanout: GraphNodeContainer = GraphNodeWrapper::new(
        FanOut::new(),
        children::boxed::Children::new(voices.into_boxed_slice()),
    )
    .into();

    let root = EventContainer::new(RootClock::new(micros, fanout));
    assert!(SCHEDULE_QUEUE.lock().enqueue(0, root).is_ok());

    //draw
    {
        let cpage = current_page.clone();
        let draw_data: Vec<_> = page_data.iter().cloned().collect();
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
        let draw = Box::new(
            move |display: &mut QuNeoDisplay, _context: &mut dyn EventEvalContext| {
                let page = cpage.get();
                //display.force_draw();
                let pages = draw_data.len();

                for p in 0..pages {
                    //indicate the current page
                    display.update(QDisplayType::Pad, p, if p == page { 127u8 } else { 0 });
                    //flash page buttons for off page sequences when they are triggered
                    //
                    //really just need these states:
                    //Off,
                    //TriggerOn (draw, schedule TriggerOff, set Off),
                    //TriggerOff (draw, set Off)
                    //
                    //could use a one shot
                    //
                    if p != page {
                        /*
                        if data.triggered.get() {
                            data.triggered.set(false);
                            display.update(QDisplayType::Pad, p, 64u8);
                            context.schedule_value(
                                TimeSched::Relative(4410),
                                &BindingSet::Bool(true, data.triggered_off.clone()),
                            );
                        }
                        if data.triggered_off.get() {
                            data.triggered_off.set(false);
                            display.update(QDisplayType::Pad, p, 0);
                        }
                        */
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
            &MIDI_QUEUE as MidiEnqueue,
            TickResched::Relative(4410),
            draw,
        );
        let draw = EventContainer::new(draw);
        assert!(SCHEDULE_QUEUE.lock().enqueue(0, draw).is_ok());
    }

    MIDI_VALUE_SOURCE.fill();
    println!("starting dispose thread");
    std::thread::spawn(|| loop {
        //midi value queue filling
        MIDI_VALUE_SOURCE.fill();
        //dispose thread, simply ditching
        while let Some(_item) = DISPOSE_SINK.dequeue() {
            /*
            println!("got dispose");
            let a = Into::<BoxEventEval>::into(item).into_any();
            if a.is::<TickedValueQueueEvent<MidiValue, MidiEnqueue>>() {
                println!("is TickedValueQueueEvent<MidiValue, ..>");
            }
            */
        }
        std::thread::sleep(std::time::Duration::from_millis(1));
    });

    let update_bpm = move |offset: f32, vel: u8| {
        let c = bpm.get() + offset * (1.0 + 5.0 * (vel as f32) / 127f32);
        bpm.set(c);
    };

    let process_callback = move |client: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
        let process_note = |on: bool, chan: u8, num: u8, vel: u8| {
            match chan {
                15 => {
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
                                67 => len_select_shift.set(on),
                                76 => mul_select_shift.set(on),
                                77 => div_select_shift.set(on),
                                78 => {
                                    if on {
                                        update_bpm(1f32, vel);
                                    }
                                }
                                79 => {
                                    if on {
                                        update_bpm(-1f32, vel);
                                    }
                                }
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

        let now = ex.tick_next();
        ex.run(ps.n_frames() as usize, client.sample_rate() as usize);

        {
            let mut out_p = midi_out.writer(ps);
            let mut write_midi = |tick: u32, bytes: &[u8]| {
                let _ = out_p.write(&jack::RawMidi { time: tick, bytes });
            };
            let mut q = MIDI_QUEUE.lock();
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
        let c = JACK_CONNECTION_COUNT.get();
        if are_connected {
            JACK_CONNECTION_COUNT.set(c + 1);
        } else if c > 0 {
            JACK_CONNECTION_COUNT.set(c - 1);
        }
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
