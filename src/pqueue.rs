extern crate alloc;

pub trait TickPriorityEnqueue<T>: Send {
    fn enqueue(&mut self, tick: usize, value: T) -> Result<(), T>;
}

pub trait TickPriorityDequeue<T>: Send {
    fn dequeue_lt(&mut self, tick: usize) -> Option<(usize, T)>;
}

//XXX is there a better way to setup Q below so that this doesn't need to be implemented?
impl<T> TickPriorityEnqueue<T> for alloc::sync::Arc<spin::Mutex<dyn TickPriorityEnqueue<T>>>
where
    T: Send,
{
    fn enqueue(&mut self, tick: usize, value: T) -> Result<(), T> {
        self.lock().enqueue(tick, value)
    }
}

impl<T> TickPriorityDequeue<T> for alloc::sync::Arc<spin::Mutex<dyn TickPriorityDequeue<T>>>
where
    T: Send,
{
    fn dequeue_lt(&mut self, tick: usize) -> Option<(usize, T)> {
        self.lock().dequeue_lt(tick)
    }
}
