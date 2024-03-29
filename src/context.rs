//! Context implementations
use crate::{event::*, pqueue::TickPriorityEnqueue, tick::*, Float};

pub struct RootContext<'a, E> {
    tick: usize,
    ticks_per_second: usize,
    schedule: &'a mut dyn TickPriorityEnqueue<E>,
}

pub struct ChildContext<'a, E> {
    parent: &'a mut dyn EventEvalContext<E>,
    parent_tick_offset: isize,
    context_tick: usize,
    context_ticks_per_second: usize,
    context_tick_period_micros: Float,
}

impl<'a, E> RootContext<'a, E> {
    pub fn new(
        tick: usize,
        ticks_per_second: usize,
        schedule: &'a mut dyn TickPriorityEnqueue<E>,
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

impl<'a, E> EventSchedule<E> for RootContext<'a, E> {
    fn event_try_schedule(&mut self, tick: TickSched, event: E) -> Result<(), E> {
        //in the root, context and absolute are the same
        let tick = match tick {
            TickSched::Absolute(t) | TickSched::ContextAbsolute(t) => t,
            TickSched::Relative(o) | TickSched::ContextRelative(o) => offset_tick(self.tick, o),
        };
        self.schedule.try_enqueue(tick, event)
    }
}

impl<'a, E> TickContext for RootContext<'a, E> {
    fn tick_now(&self) -> usize {
        self.tick
    }
    fn ticks_per_second(&self) -> usize {
        self.ticks_per_second
    }
}

impl<'a, E> ChildContext<'a, E> {
    pub fn new(
        parent: &'a mut dyn EventEvalContext<E>,
        parent_tick_offset: isize,
        context_tick: usize,
        context_tick_period_micros: Float,
    ) -> Self {
        //XXX TEST
        let ps = 1.0e6 / context_tick_period_micros;
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

impl<'a, E> EventSchedule<E> for ChildContext<'a, E> {
    fn event_try_schedule(&mut self, tick: TickSched, event: E) -> Result<(), E> {
        //XXX TODO TRANSLATE TO CONTEXT TIME IF NEEDED
        self.parent.event_try_schedule(tick, event)
    }
}

impl<'a, E> TickContext for ChildContext<'a, E> {
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
    fn tick_period_micros(&self) -> Float {
        self.parent.tick_period_micros()
    }
    fn context_tick_period_micros(&self) -> Float {
        self.context_tick_period_micros
    }
}

/*
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

    impl<E> EventSchedule<E> for TestContext {
        fn event_try_schedule(&mut self, _tick: TickSched, _event: E) -> Result<(), E> {
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
*/
