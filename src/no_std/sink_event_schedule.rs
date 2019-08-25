#![no_main]

use crate::event::*;
use crate::time::*;
use core::marker::PhantomData;
use core::ops::DerefMut;

pub struct SinkEventSchedule<T>
where
    T: DerefMut<Target = dyn SinkEventEval<T>>,
{
    now: usize,
    phantom: PhantomData<T>,
}

impl<T> SinkEventSchedule<T>
where
    T: DerefMut<Target = dyn SinkEventEval<T>>,
{
    pub fn new() -> Self {
        Self {
            now: 0,
            phantom: PhantomData,
        }
    }
}

impl<T> ScheduleSinkEvent<T> for SinkEventSchedule<T>
where
    T: DerefMut<Target = dyn SinkEventEval<T>>,
{
    fn schedule_event(&mut self, time: TimeSched, event: T) -> Result<(), core::fmt::Error> {
        Ok(())
    }
}
