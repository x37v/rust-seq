use crate::binding::ParamBindingGet;
use crate::event::{EventContainer, EventEvalContext};
use crate::graph::GraphLeafExec;
use crate::tick::{TickResched, TickSched};

use crate::event::ticked_value_queue::TickedValueQueueEvent;
use crate::item_source::ItemSource;
use crate::midi::MidiValue;
use crate::pqueue::TickPriorityEnqueue;
extern crate alloc;
use alloc::boxed::Box;

pub type TickedMidiValueEvent<Enqueue> = TickedValueQueueEvent<MidiValue, Enqueue>;

/// Graph leaf that schedules a midi note at context 'now' with the duration given.
pub struct MidiNote<Chan, Note, Dur, OnVel, OffVel, MidiValueQueue, Source>
where
    Chan: ParamBindingGet<u8>,
    Note: ParamBindingGet<u8>,
    Dur: ParamBindingGet<TickResched>,
    OnVel: ParamBindingGet<u8>,
    OffVel: ParamBindingGet<u8>,
    MidiValueQueue: 'static + TickPriorityEnqueue<MidiValue> + Clone,
    Source: 'static
        + ItemSource<TickedMidiValueEvent<MidiValueQueue>, Box<TickedMidiValueEvent<MidiValueQueue>>>,
{
    chan: Chan,
    note: Note,
    dur: Dur,
    on_vel: OnVel,
    off_vel: OffVel,
    source: Source,
    queue: MidiValueQueue,
}

impl<Chan, Note, Dur, OnVel, OffVel, MidiValueQueue, Source>
    MidiNote<Chan, Note, Dur, OnVel, OffVel, MidiValueQueue, Source>
where
    Chan: ParamBindingGet<u8>,
    Note: ParamBindingGet<u8>,
    Dur: ParamBindingGet<TickResched>,
    OnVel: ParamBindingGet<u8>,
    OffVel: ParamBindingGet<u8>,
    MidiValueQueue: 'static + TickPriorityEnqueue<MidiValue> + Clone,
    Source: 'static
        + ItemSource<TickedMidiValueEvent<MidiValueQueue>, Box<TickedMidiValueEvent<MidiValueQueue>>>,
{
    pub fn new(
        chan: Chan,
        note: Note,
        dur: Dur,
        on_vel: OnVel,
        off_vel: OffVel,
        source: Source,
        queue: MidiValueQueue,
    ) -> Self {
        Self {
            chan,
            note,
            dur,
            on_vel,
            off_vel,
            source,
            queue,
        }
    }
}

impl<Chan, Note, Dur, OnVel, OffVel, MidiValueQueue, Source> GraphLeafExec
    for MidiNote<Chan, Note, Dur, OnVel, OffVel, MidiValueQueue, Source>
where
    Chan: ParamBindingGet<u8>,
    Note: ParamBindingGet<u8>,
    Dur: ParamBindingGet<TickResched>,
    OnVel: ParamBindingGet<u8>,
    OffVel: ParamBindingGet<u8>,
    MidiValueQueue: 'static + TickPriorityEnqueue<MidiValue> + Clone,
    Source: 'static
        + ItemSource<TickedMidiValueEvent<MidiValueQueue>, Box<TickedMidiValueEvent<MidiValueQueue>>>,
{
    fn graph_exec(&self, context: &mut dyn EventEvalContext) {
        let chan = self.chan.get();
        let num = self.note.get();
        let dur = self.dur.get();
        let on_vel = self.on_vel.get();
        let off_vel = self.off_vel.get();

        let on = self.source.try_get(TickedValueQueueEvent::new(
            MidiValue::NoteOn {
                chan,
                num,
                vel: on_vel,
            },
            self.queue.clone(),
        ));
        let off = self.source.try_get(TickedValueQueueEvent::new(
            MidiValue::NoteOff {
                chan,
                num,
                vel: off_vel,
            },
            self.queue.clone(),
        ));

        if let Ok(off) = off {
            if let Ok(on) = on {
                let t = TickSched::ContextRelative(0);
                let ot = t.add(dur, context.as_tick_context());
                //schedule off first
                let s = context.event_schedule(ot, EventContainer::new_from_box(off));
                if let Err(_b) = s {
                    //dispose
                    //XXX report
                    #[cfg(feature = "std")]
                    println!("should dispose");
                } else {
                    let s = context.event_schedule(t, EventContainer::new_from_box(on));
                    if let Err(_b) = s {
                        //dispose
                        #[cfg(feature = "std")]
                        println!("should dispose 2");
                        //XXX report
                    }
                }
            } else {
                //XXX report
                #[cfg(feature = "std")]
                println!("cannot get on");
            }
        } else {
            //XXX report
            #[cfg(feature = "std")]
            println!("cannot get off");
        }
    }
}
