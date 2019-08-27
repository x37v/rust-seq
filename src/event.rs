use crate::time::*;
use core::ops::DerefMut;

/// Sink events are events that happen on the output.
/// They potentially generate other events, they also may hold other events and gate their actual
/// output.

/// trait for evaluating SinkEvents
pub trait SinkEventEval<Container>
where
    Container: DerefMut<Target = dyn SinkEventEval<Container>>,
{
    fn sink_eval(&mut self, context: &mut dyn ScheduleSinkContext<Container>);
}

/// Interface to schedule SinkEvents
/// most likely: T: DerefMut<dyn SinkEventEval>
pub trait ScheduleSinkEvent<Container>
where
    Container: DerefMut<Target = dyn SinkEventEval<Container>>,
{
    fn schedule_event(&mut self, time: TimeSched, event: Container)
        -> Result<(), core::fmt::Error>;
}

pub trait ScheduleSinkContext<Container>: ScheduleSinkEvent<Container> + TimeContext
where
    Container: DerefMut<Target = dyn SinkEventEval<Container>>,
{
}

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
