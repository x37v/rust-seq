use crate::event::*;
use crate::pqueue::TickPriorityEnqueue;
use crate::tick::*;

pub struct RootContext<'a> {
    tick: usize,
    ticks_per_second: usize,
    schedule: &'a mut dyn TickPriorityEnqueue<EventContainer>,
}

pub struct ChildContext<'a> {
    parent: &'a mut dyn EventEvalContext,
    parent_tick_offset: isize,
    context_tick: usize,
    context_ticks_per_second: usize,
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
        tick: TickSched,
        event: EventContainer,
    ) -> Result<(), EventContainer> {
        //in the root, context and absolute are the same
        let tick = match tick {
            TickSched::Absolute(t) | TickSched::ContextAbsolute(t) => t,
            TickSched::Relative(o) | TickSched::ContextRelative(o) => offset_tick(self.tick, o),
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
        //XXX TEST
        let ps = 1e6f32 / context_tick_period_micros;
        Self {
            parent,
            parent_tick_offset,
            context_tick,
            context_ticks_per_second: ps as usize,
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
        tick: TickSched,
        event: EventContainer,
    ) -> Result<(), EventContainer> {
        //XXX TODO TRANSLATE TO CONTEXT TIME IF NEEDED
        self.parent.event_schedule(tick, event)
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
    fn context_ticks_per_second(&self) -> usize {
        self.context_ticks_per_second
    }
    fn tick_period_micros(&self) -> f32 {
        self.parent.tick_period_micros()
    }
    fn context_tick_period_micros(&self) -> f32 {
        self.context_tick_period_micros
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    pub struct TestContext {
        tick: usize,
        ticks_per_second: usize,
    }

    impl TestContext {
        pub fn new(tick: usize, ticks_per_second: usize) -> Self {
            Self {
                tick,
                ticks_per_second,
            }
        }

        pub fn set_tick(&mut self, tick: usize) {
            self.tick = tick;
        }
    }
    impl EventSchedule for TestContext {
        fn event_schedule(
            &mut self,
            _tick: TickSched,
            _event: EventContainer,
        ) -> Result<(), EventContainer> {
            Ok(())
        }
    }

    impl TickContext for TestContext {
        fn tick_now(&self) -> usize {
            self.tick
        }
        fn ticks_per_second(&self) -> usize {
            self.ticks_per_second
        }
    }
}
