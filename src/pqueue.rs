extern crate alloc;
use core::cmp::Ordering;

pub struct TickItem<T> {
    tick: usize,
    item: T,
}

impl<T> TickItem<T> {
    pub fn tick(&self) -> usize {
        self.tick
    }
}

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

impl<T> TickPriorityEnqueue<T> for &'static spin::Mutex<dyn TickPriorityEnqueue<T>>
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

impl<T> TickPriorityDequeue<T> for &'static spin::Mutex<dyn TickPriorityDequeue<T>>
where
    T: Send,
{
    fn dequeue_lt(&mut self, tick: usize) -> Option<(usize, T)> {
        self.lock().dequeue_lt(tick)
    }
}

impl<T> core::convert::From<(usize, T)> for TickItem<T> {
    fn from(item: (usize, T)) -> Self {
        Self {
            tick: item.0,
            item: item.1,
        }
    }
}

impl<T> core::convert::Into<(usize, T)> for TickItem<T> {
    fn into(self) -> (usize, T) {
        (self.tick, self.item)
    }
}

impl<T> Ord for TickItem<T>
where
    T: Ord,
{
    fn cmp(&self, other: &Self) -> Ordering {
        match self.tick.cmp(&other.tick) {
            Ordering::Less => Ordering::Less,
            Ordering::Greater => Ordering::Greater,
            Ordering::Equal => self.item.cmp(&other.item),
        }
    }
}

impl<T> PartialOrd for TickItem<T>
where
    T: Ord,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> PartialEq for TickItem<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        if self.tick.eq(&other.tick) {
            self.item.eq(&other.item)
        } else {
            false
        }
    }
}

impl<T> Eq for TickItem<T> where T: Eq {}