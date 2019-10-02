use crate::event::{EventEval, EventEvalContext};
use crate::pqueue::TickPriorityEnqueue;
use crate::time::TimeResched;

/// An event that pushes a value into a queue with tick = context.time_now()
///
/// This is most likely to be used for output events like, Midi::NoteOn(_,_,_),
/// Trigger(index) etc..
pub struct TickedValueQueueEvent<T, Q>
where
    T: 'static + Send + Copy,
    Q: 'static + TickPriorityEnqueue<T>,
{
    value: T,
    queue: Q,
}

impl<T, Q> TickedValueQueueEvent<T, Q>
where
    T: 'static + Send + Copy,
    Q: 'static + TickPriorityEnqueue<T>,
{
    pub fn new(value: T, queue: Q) -> Self {
        Self { value, queue }
    }
}

impl<T, Q> EventEval for TickedValueQueueEvent<T, Q>
where
    T: 'static + Send + Copy,
    Q: 'static + TickPriorityEnqueue<T>,
{
    fn event_eval(&mut self, context: &mut dyn EventEvalContext) -> TimeResched {
        let t = context.time_now();
        let r = self.queue.enqueue(t, self.value);
        if r.is_err() {
            //XXX report
        }
        TimeResched::None
    }
}

#[cfg(test)]
mod tests {
    extern crate alloc;
    use super::*;
    use crate::event::EventEvalAny;
    use alloc::sync::Arc;
    use spin::Mutex;

    struct TestQueue;

    impl<T> TickPriorityEnqueue<T> for TestQueue
    where
        T: Send,
    {
        fn enqueue(&mut self, _tick: usize, _value: T) -> Result<(), T> {
            Ok(())
        }
    }

    #[test]
    pub fn can_build() {
        type Queue = Arc<Mutex<dyn TickPriorityEnqueue<usize>>>;
        let q: Queue = Arc::new(Mutex::new(TestQueue));
        let e = alloc::boxed::Box::new(TickedValueQueueEvent::new(1usize, q));
        let a = e.into_any();
        assert!(a.is::<TickedValueQueueEvent<usize, Queue>>());
    }
}
