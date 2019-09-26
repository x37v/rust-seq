use crate::time::*;

extern crate alloc;
use alloc::boxed::Box;
pub type EventContainer = Box<dyn EventEval>;

/// Events potentially generate other events, they also may hold other events and gate their actual
/// output.

/// trait for evaluating Events
pub trait EventEval: Send + core::any::Any {
    fn event_eval(&mut self, context: &mut dyn EventEvalContext);
    fn into_any(self: Box<Self>) -> Box<dyn core::any::Any>;
}

/// Interface to schedule Events
pub trait EventSchedule {
    fn event_schedule(
        &mut self,
        time: TimeSched,
        event: EventContainer,
    ) -> Result<(), EventContainer>;
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

#[cfg(test)]
mod tests {
    use super::*;

    struct Test;
    impl EventEval for Test {
        fn event_eval(&mut self, context: &mut dyn EventEvalContext) {}
        fn into_any(self: Box<Self>) -> Box<dyn core::any::Any> {
            self
        }
    }

}
