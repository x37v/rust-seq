use crate::time::*;
use core::cmp::Ordering;

pub mod ticked_value_queue;

extern crate alloc;
use alloc::boxed::Box;
pub struct EventContainer(Box<dyn EventEvalAny>);

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

impl EventContainer {
    pub fn new(item: Box<dyn EventEvalAny>) -> Self {
        Self(item)
    }
}

impl EventEval for EventContainer {
    fn event_eval(&mut self, context: &mut dyn EventEvalContext) -> TimeResched {
        self.0.event_eval(context)
    }
}

impl Ord for EventContainer {
    fn cmp(&self, other: &Self) -> Ordering {
        let left: *const _ = self.0.as_ref();
        let right: *const _ = other.0.as_ref();
        left.cmp(&right)
    }
}

impl PartialOrd for EventContainer {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for EventContainer {
    fn eq(&self, _other: &Self) -> bool {
        false //box, never equal
    }
}

impl Eq for EventContainer {}

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
