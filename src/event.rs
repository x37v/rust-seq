use crate::time::*;

pub mod ticked_value_queue;

extern crate alloc;
use alloc::boxed::Box;
pub type EventContainer = Box<dyn EventEvalAny>;

/// Events potentially generate other events, they also may hold other events and gate their actual
/// output.

/// trait for evaluating Events
pub trait EventEval: Send {
    fn event_eval(&mut self, context: &mut dyn EventEvalContext) -> TimeResched;
}

/// helper trait that we use so we can downcast
pub trait EventEvalAny: EventEval + core::any::Any {
    fn into_any(self: Box<Self>) -> Box<dyn core::any::Any>;
}

impl<T> EventEvalAny for T
where
    T: 'static + EventEval,
{
    fn into_any(self: Box<Self>) -> Box<dyn core::any::Any> {
        self
    }
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

impl<T> EventEvalContext for T where T: EventSchedule + TimeContext {}

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
        fn event_eval(&mut self, _context: &mut dyn EventEvalContext) -> TimeResched {
            TimeResched::None
        }
    }

}
