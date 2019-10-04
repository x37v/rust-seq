use crate::event::*;
use crate::pqueue::TickPriorityEnqueue;
use crate::time::*;

pub struct RootContext<'a> {
    tick: usize,
    ticks_per_second: usize,
    schedule: &'a mut dyn TickPriorityEnqueue<EventContainer>,
}

pub struct ChildContext<'a> {
    parent: &'a mut dyn EventEvalContext,
    parent_tick_offset: isize,
    context_tick: usize,
    context_tick_period_micros: f32,
}

impl<'a> RootContext<'a> {
    pub fn new(
        tick: usize,
        ticks_per_second: usize,
        schedule: &'a mut dyn TickPriorityEnqueue<EventContainer>,
    ) -> Self {
        Self {
            tick,
            ticks_per_second,
            schedule,
        }
    }

    pub fn update_tick(&mut self, tick: usize) {
        self.tick = tick;
    }
}

impl<'a> EventSchedule for RootContext<'a> {
    fn event_schedule(
        &mut self,
        time: TimeSched,
        event: EventContainer,
    ) -> Result<(), EventContainer> {
        //in the root, context and absolute are the same
        let tick = match time {
            TimeSched::Absolute(t) | TimeSched::ContextAbsolute(t) => t,
            TimeSched::Relative(o) | TimeSched::ContextRelative(o) => offset_tick(self.tick, o),
        };
        self.schedule.enqueue(tick, event)
    }
}

impl<'a> TickContext for RootContext<'a> {
    fn tick_now(&self) -> usize {
        self.tick
    }
    fn ticks_per_second(&self) -> usize {
        self.ticks_per_second
    }
}

impl<'a> ChildContext<'a> {
    pub fn new(
        parent: &'a mut dyn EventEvalContext,
        parent_tick_offset: isize,
        context_tick: usize,
        context_tick_period_micros: f32,
    ) -> Self {
        Self {
            parent,
            parent_tick_offset,
            context_tick,
            context_tick_period_micros,
        }
    }

    pub fn update_parent_offset(&mut self, offset: isize) {
        self.parent_tick_offset = offset;
    }

    pub fn update_context_tick(&mut self, tick: usize) {
        self.context_tick = tick;
    }
}

impl<'a> EventSchedule for ChildContext<'a> {
    fn event_schedule(
        &mut self,
        time: TimeSched,
        event: EventContainer,
    ) -> Result<(), EventContainer> {
        //XXX TODO TRANSLATE TO CONTEXT TIME IF NEEDED
        self.parent.event_schedule(time, event)
    }
}

impl<'a> TickContext for ChildContext<'a> {
    fn tick_now(&self) -> usize {
        offset_tick(self.parent.tick_now(), self.parent_tick_offset)
    }
    fn ticks_per_second(&self) -> usize {
        self.parent.ticks_per_second()
    }
    fn context_tick_now(&self) -> usize {
        self.context_tick
    }
    fn tick_period_micros(&self) -> f32 {
        self.parent.tick_period_micros()
    }
    fn context_tick_period_micros(&self) -> f32 {
        self.context_tick_period_micros
    }
}
