use jack;
use std::io;

use sched::event::ticked_value_queue::TickedValueQueueEvent;
use sched::event::*;
use sched::item_sink::ItemSink;
use sched::midi::*;
use sched::pqueue::*;
use sched::schedule::ScheduleExecutor;

use heapless::binary_heap::{BinaryHeap, Min};
use heapless::consts::*;

struct Sink;

struct ScheduleQueue(BinaryHeap<TickItem<EventContainer>, U8, Min>);
struct MidiQueue(BinaryHeap<TickItem<MidiValue>, U8, Min>);

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

impl ItemSink<EventContainer> for Sink {
    fn try_put(&mut self, item: EventContainer) -> Result<(), EventContainer> {
        Ok(())
    }
}

static DISPOSE_SINK: spin::Mutex<Sink> = spin::Mutex::new(Sink {});
static SCHEDULE_QUEUE: spin::Mutex<ScheduleQueue> =
    spin::Mutex::new(ScheduleQueue(BinaryHeap(heapless::i::BinaryHeap::new())));
static MIDI_QUEUE: spin::Mutex<MidiQueue> =
    spin::Mutex::new(MidiQueue(BinaryHeap(heapless::i::BinaryHeap::new())));

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
        &DISPOSE_SINK as &'static spin::Mutex<dyn ItemSink<EventContainer>>,
        &SCHEDULE_QUEUE as &'static spin::Mutex<dyn TickPriorityDequeue<EventContainer>>,
        &SCHEDULE_QUEUE as &'static spin::Mutex<dyn TickPriorityEnqueue<EventContainer>>,
    );

    let note_on = EventContainer::new(Box::new(TickedValueQueueEvent::new(
        MidiValue::NoteOn {
            chan: 0,
            num: 64,
            vel: 127,
        },
        &MIDI_QUEUE as &spin::Mutex<dyn TickPriorityEnqueue<MidiValue>>,
    )));
    let note_off = EventContainer::new(Box::new(TickedValueQueueEvent::new(
        MidiValue::NoteOff {
            chan: 0,
            num: 64,
            vel: 127,
        },
        &MIDI_QUEUE as &spin::Mutex<dyn TickPriorityEnqueue<MidiValue>>,
    )));

    let off = 44100usize * 10usize;
    assert!(SCHEDULE_QUEUE
        .lock()
        .enqueue(off + 44100usize * 2usize, note_off)
        .is_ok());
    assert!(SCHEDULE_QUEUE
        .lock()
        .enqueue(off + 44100usize, note_on)
        .is_ok());

    let process_callback = move |client: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
        //read in midi
        for m in midi_in.iter(ps) {}

        let now = ex.tick_next();
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

        {
            let mut q = MIDI_QUEUE.lock();
            let next = ex.tick_next();
            while let Some((t, midi)) = q.dequeue_lt(next) {
                let time = (if t < now { now } else { t } - now) as u32;
                write_midi_value(time, &midi);
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
        _are_connected: bool,
    ) {
        /*
        let c = self.connection_count.get();
        if are_connected {
            self.connection_count.set(1 + c);
        } else if c > 0 {
            self.connection_count.set(c - 1);
        }
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
