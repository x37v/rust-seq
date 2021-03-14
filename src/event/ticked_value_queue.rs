use crate::event::{EventEval, EventEvalContext};
use crate::pqueue::TickPriorityEnqueue;
use crate::tick::TickResched;

/// An event that pushes a value into a queue with tick = context.tick_now()
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
    fn event_eval(&mut self, context: &mut dyn EventEvalContext) -> TickResched {
        let t = context.tick_now();
        let r = self.queue.enqueue(t, self.value);
        if r.is_err() {
            //XXX report
        }
        TickResched::None
    }
}

#[cfg(test)]
mod tests {
    extern crate alloc;
    use super::*;
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
        let _: Queue = Arc::new(Mutex::new(TestQueue));
    }
}
