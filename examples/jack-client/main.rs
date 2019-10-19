use jack;
use std::io;

extern crate alloc;
mod quneo_display;

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
    clock_ratio::ClockRatio, node_wrapper::GraphNodeWrapper, root_clock::RootClock,
    step_seq::StepSeq,
};

use sched::binding::*;

use core::mem::{self, MaybeUninit};

use heapless::binary_heap::{BinaryHeap, Min};
use heapless::consts::*;
use heapless::mpmc::Q64;

use core::sync::atomic::{AtomicBool, AtomicU8, AtomicUsize};

pub struct ScheduleQueue(BinaryHeap<TickItem<EventContainer>, U8, Min>);
pub struct MidiQueue(BinaryHeap<TickItem<MidiValue>, U8, Min>);
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

    let mut ex = ScheduleExecutor::new(
        &DISPOSE_SINK,
        &SCHEDULE_QUEUE as &'static spin::Mutex<dyn TickPriorityDequeue<EventContainer>>,
        &SCHEDULE_QUEUE as &'static spin::Mutex<dyn TickPriorityEnqueue<EventContainer>>,
    );

    /*
    let note_on = EventContainer::new(TickedValueQueueEvent::new(
        MidiValue::NoteOn {
            chan: 0,
            num: 64,
            vel: 127,
        },
        &MIDI_QUEUE as MidiEnqueue,
    ));
    let note_off = EventContainer::new(TickedValueQueueEvent::new(
        MidiValue::NoteOff {
            chan: 0,
            num: 64,
            vel: 127,
        },
        &MIDI_QUEUE as MidiEnqueue,
    ));

    let off = 44100usize * 10usize;
    assert!(SCHEDULE_QUEUE
        .lock()
        .enqueue(off + 44100usize * 2usize, note_off)
        .is_ok());
    assert!(SCHEDULE_QUEUE
        .lock()
        .enqueue(off + 44100usize, note_on)
        .is_ok());
        */

    let ppq = 980usize;
    let mul = AtomicU8::new(1).into_arc();
    let div = AtomicU8::new(1).into_arc();
    let steps = AtomicUsize::new(16).into_arc();
    let step_ticks = AtomicUsize::new(ppq / 4usize).into_arc();
    let step_cur = AtomicUsize::new(16).into_arc();

    let clock_binding: Arc<Mutex<dyn bpm::Clock>> = bpm::ClockData::new(100.0, ppq).into_alock();
    let _bpm = bpm::ClockBPMBinding(clock_binding.clone()).into_arc();
    let _ppq = bpm::ClockPPQBinding(clock_binding.clone()).into_arc();
    let micros: Arc<dyn ParamBindingGet<f32>> =
        bpm::ClockPeriodMicroBinding(clock_binding.clone()).into_arc();

    //XXX could try `arr_macro` for this
    let gates: Arc<[Arc<AtomicBool>]> = Arc::new([
        Arc::new(AtomicBool::new(true)),
        Arc::new(AtomicBool::new(false)),
        Arc::new(AtomicBool::new(false)),
        Arc::new(AtomicBool::new(false)),
        Arc::new(AtomicBool::new(true)),
        Arc::new(AtomicBool::new(false)),
        Arc::new(AtomicBool::new(false)),
        Arc::new(AtomicBool::new(false)),
        Arc::new(AtomicBool::new(true)),
        Arc::new(AtomicBool::new(false)),
        Arc::new(AtomicBool::new(false)),
        Arc::new(AtomicBool::new(false)),
        Arc::new(AtomicBool::new(true)),
        Arc::new(AtomicBool::new(false)),
        Arc::new(AtomicBool::new(false)),
        Arc::new(AtomicBool::new(false)),
    ]);
    let gatesg: Arc<[Arc<dyn ParamBindingGet<bool>>]> = Arc::new([
        gates[0].clone() as Arc<dyn ParamBindingGet<bool>>,
        gates[1].clone() as Arc<dyn ParamBindingGet<bool>>,
        gates[2].clone() as Arc<dyn ParamBindingGet<bool>>,
        gates[3].clone() as Arc<dyn ParamBindingGet<bool>>,
        gates[4].clone() as Arc<dyn ParamBindingGet<bool>>,
        gates[5].clone() as Arc<dyn ParamBindingGet<bool>>,
        gates[6].clone() as Arc<dyn ParamBindingGet<bool>>,
        gates[7].clone() as Arc<dyn ParamBindingGet<bool>>,
        gates[8].clone() as Arc<dyn ParamBindingGet<bool>>,
        gates[9].clone() as Arc<dyn ParamBindingGet<bool>>,
        gates[10].clone() as Arc<dyn ParamBindingGet<bool>>,
        gates[11].clone() as Arc<dyn ParamBindingGet<bool>>,
        gates[12].clone() as Arc<dyn ParamBindingGet<bool>>,
        gates[13].clone() as Arc<dyn ParamBindingGet<bool>>,
        gates[14].clone() as Arc<dyn ParamBindingGet<bool>>,
        gates[15].clone() as Arc<dyn ParamBindingGet<bool>>,
    ]);

    //root -> ratio -> step_seq ---(nchild index bind)--> step_gate -> note

    let ratio = ClockRatio::new(
        mul as Arc<dyn ParamBindingGet<_>>,
        div as Arc<dyn ParamBindingGet<_>>,
    );

    let seq = StepSeq::new(
        step_ticks as Arc<dyn ParamBindingGet<_>>,
        steps as Arc<dyn ParamBindingGet<_>>,
    );

    let step_cur_bind = IndexChildContainer::new(bindstore::BindStoreIndexChild::new(
        step_cur.clone() as Arc<dyn ParamBindingSet<usize>>,
    ));

    let step_gate = ops::GetIndexed::new(
        gatesg.clone(),
        step_cur.clone() as Arc<dyn ParamBindingGet<usize>>,
    )
    .into_alock();

    let step_gate = gate::Gate::new(step_gate as Arc<Mutex<dyn ParamBindingGet<bool>>>);

    let note = midi::MidiNote::new(
        &0,
        &64,
        &TickResched::ContextRelative(1),
        &127,
        &127,
        &MIDI_VALUE_SOURCE,
        &MIDI_QUEUE as MidiEnqueue,
    );

    let note: GraphNodeContainer = GraphNodeWrapper::new(note, children::empty::Children).into();

    let step_gate: GraphNodeContainer =
        GraphNodeWrapper::new(step_gate, children::boxed::Children::new(Box::new([note]))).into();

    let ichild = children::boxed::IndexChildren::new(Box::new([step_cur_bind]));

    let seq: GraphNodeContainer =
        GraphNodeWrapper::new(seq, children::nchild::ChildWrapper::new(step_gate, ichild)).into();

    let ratio: GraphNodeContainer =
        GraphNodeWrapper::new(ratio, children::boxed::Children::new(Box::new([seq]))).into();

    let root = EventContainer::new(RootClock::new(micros, ratio));
    assert!(SCHEDULE_QUEUE.lock().enqueue(0, root).is_ok());

    MIDI_VALUE_SOURCE.fill();
    std::thread::spawn(|| loop {
        //midi value queue filling
        MIDI_VALUE_SOURCE.fill();
        //dispose thread, simply ditching
        if let Some(_item) = DISPOSE_SINK.dequeue() {
            /*
            println!("got dispose");
            let a = Into::<BoxEventEval>::into(item).into_any();
            if a.is::<TickedValueQueueEvent<MidiValue, MidiEnqueue>>() {
                println!("is TickedValueQueueEvent<MidiValue, ..>");
            }
            */
        } else {
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    });

    let process_callback = move |client: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
        //read in midi
        for _m in midi_in.iter(ps) {}

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
