use crate::event::{EventEval, EventEvalContext};

pub trait TickPriorityQueue<T>: Send
where
    T: Send,
{
    fn queue(&mut self, tick: usize, value: T) -> Result<(), T>;
}

/// An event that pushes a value into a queue with tick = context.time_now()
///
/// This is most likely to be used for output events like, Midi::NoteOn(_,_,_),
/// Trigger(index) etc..
pub struct TickedValueQueueEvent<T, Q>
where
    T: 'static + Send + Copy,
    Q: 'static + TickPriorityQueue<T>,
{
    value: T,
    queue: Q,
}

impl<T, Q> TickedValueQueueEvent<T, Q>
where
    T: 'static + Send + Copy,
    Q: 'static + TickPriorityQueue<T>,
{
    pub fn new(value: T, queue: Q) -> Self {
        Self { value, queue }
    }
}

impl<T, Q> EventEval for TickedValueQueueEvent<T, Q>
where
    T: 'static + Send + Copy,
    Q: 'static + TickPriorityQueue<T>,
{
    fn event_eval(&mut self, context: &mut dyn EventEvalContext) {
        let t = context.time_now();
        let r = self.queue.queue(t, self.value);
        if r.is_err() {
            //XXX report
        }
    }

    fn into_any(self: Box<Self>) -> Box<dyn core::any::Any> {
        self
    }
}
