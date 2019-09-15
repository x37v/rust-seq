use crate::time::*;
use core::ops::DerefMut;

cfg_if::cfg_if! {
    if #[cfg(feature = "std")] {
        //XXX would like to use ChannelDropBox ...
        type EventContainer = Box<dyn EventEval>;
    } else {
        type EventContainer = &'static mut dyn EventEval;
    }
}

/// Events potentially generate other events, they also may hold other events and gate their actual
/// output.

/// trait for evaluating Events
pub trait EventEval: Send {
    fn event_eval(&mut self, context: &mut dyn EventEvalContext);
}

/// Interface to schedule Events
pub trait EventSchedule {
    fn event_schedule(
        &mut self,
        time: TimeSched,
        event: EventContainer,
    ) -> Result<(), core::fmt::Error>;
}

pub trait EventEvalContext: EventSchedule + TimeContext {}

/*
 *
 *  example, scheduling midi on/off
 *
 *  some fallible function
 *
 *  let on = midi_item_source.try_pop()?;
 *  on.set_note_on(..details.., midi_item_source.clone())?;
 *  scheduler.schedule_event(start, on)?;
 *
 *  the note on will grab a note off node,
 *  if that fails, it won't push to the output
 *
 *
 *
 */
