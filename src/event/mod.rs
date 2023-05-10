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

pub trait EventEval<E, U> {
    fn event_eval(
        &mut self,
        context: &mut dyn EventEvalContext<E>,
        user_data: &mut U,
    ) -> TickResched;
}

#[cfg(feature = "with_alloc")]
pub mod boxed {
    extern crate alloc;
    use super::*;
    use core::cmp::Ordering;

    pub struct EventContainer<U> {
        inner: alloc::boxed::Box<dyn EventEval<EventContainer<U>, U>>,
    }

    impl<U> EventContainer<U> {
        pub fn new(event: alloc::boxed::Box<dyn EventEval<Self, U>>) -> Self {
            Self { inner: event }
        }
    }

    impl<U> EventEval<EventContainer<U>, U> for EventContainer<U> {
        fn event_eval(
            &mut self,
            context: &mut dyn EventEvalContext<EventContainer<U>>,
            user_data: &mut U,
        ) -> TickResched {
            self.inner.event_eval(context, user_data)
        }
    }

    //TODO drop, push to queue
    impl<U> Ord for EventContainer<U> {
        fn cmp(&self, other: &Self) -> Ordering {
            let left: *const _ = self.inner.as_ref();
            let right: *const _ = other.inner.as_ref();
            left.cmp(&right)
        }
    }

    impl<U> PartialOrd for EventContainer<U> {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }

    impl<U> PartialEq for EventContainer<U> {
        fn eq(&self, _other: &Self) -> bool {
            false //box, never equal
        }
    }

    impl<U> Eq for EventContainer<U> {}
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
