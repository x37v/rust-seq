#![no_main]

use crate::event::*;
use crate::time::*;
use core::marker::PhantomData;
use core::ops::DerefMut;

pub struct Schedule {
    now: usize,
}

impl Schedule {
    pub fn new() -> Self {
        Self { now: 0 }
    }
}

impl EventSchedule for Schedule {
    fn event_schedule(
        &mut self,
        time: TimeSched,
        event: EventContainer,
    ) -> Result<(), core::fmt::Error> {
        Ok(())
    }
}
