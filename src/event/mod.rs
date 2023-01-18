//! Events and event scheduling
use crate::tick::*;

pub mod midi;

pub trait EventSchedule<E> {
    /// Try to schedule the event at the given tick.
    fn event_try_schedule(&mut self, tick: TickSched, event: E) -> Result<(), E>;
}

pub trait EventEvalContext<E>: EventSchedule<E> + TickContext {
    fn as_tick_context(&self) -> &dyn TickContext;
    fn as_event_schedule(&mut self) -> &mut dyn EventSchedule<E>;
}

pub trait EventEval<E> {
    fn event_eval(&mut self, context: &mut dyn EventEvalContext<E>) -> TickResched;
}

#[cfg(feature = "with_alloc")]
pub mod boxed {
    extern crate alloc;
    use super::*;
    use core::cmp::Ordering;

    pub struct EventContainer {
        inner: alloc::boxed::Box<dyn EventEval<EventContainer>>,
    }

    impl EventContainer {
        pub fn new(event: alloc::boxed::Box<dyn EventEval<Self>>) -> Self {
            Self { inner: event }
        }
    }

    impl EventEval<EventContainer> for EventContainer {
        fn event_eval(
            &mut self,
            context: &mut dyn EventEvalContext<EventContainer>,
        ) -> TickResched {
            self.inner.event_eval(context)
        }
    }

    //TODO drop, push to queue
    impl Ord for EventContainer {
        fn cmp(&self, other: &Self) -> Ordering {
            let left: *const _ = self.inner.as_ref();
            let right: *const _ = other.inner.as_ref();
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
}

impl<T, E> EventEvalContext<E> for T
where
    T: EventSchedule<E> + TickContext,
{
    fn as_tick_context(&self) -> &dyn TickContext {
        self
    }
    fn as_event_schedule(&mut self) -> &mut dyn EventSchedule<E> {
        self
    }
}
