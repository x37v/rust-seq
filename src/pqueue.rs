extern crate alloc;
use core::cmp::Ordering;
use core::marker::PhantomData;

pub struct TickItem<T, OrdOrder = NormalOrd> {
    tick: usize,
    item: T,
    _ord: PhantomData<OrdOrder>,
}

/// markers for ordering, work the fact that BinaryHeap is a max heap, so we reverse Ord for
/// TickItem when using it.
pub struct NormalOrd;
pub struct ReverseOrd;

#[cfg(feature = "std")]
/// A queue that uses `std::collections::BinaryHeap` internally, with capacity so it won't
/// allocate.
pub struct BinaryHeapQueue<T>(std::collections::BinaryHeap<TickItem<T, ReverseOrd>>);

impl<T, OrdOrder> TickItem<T, OrdOrder> {
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

#[cfg(feature = "std")]
impl<T> TickPriorityEnqueue<T> for BinaryHeapQueue<T>
where
    T: Send + Ord,
{
    fn enqueue(&mut self, tick: usize, value: T) -> Result<(), T> {
        //don't allocate
        if self.0.len() >= self.0.capacity() {
            Err(value)
        } else {
            self.0.push((tick, value).into());
            Ok(())
        }
    }
}

#[cfg(feature = "std")]
impl<T> TickPriorityDequeue<T> for BinaryHeapQueue<T>
where
    T: Send + Ord,
{
    fn dequeue_lt(&mut self, tick: usize) -> Option<(usize, T)> {
        if let Some(t) = self.0.peek() {
            if t.tick() < tick {
                self.0.pop().map(|v| v.into())
            } else {
                None
            }
        } else {
            None
        }
    }
}

#[cfg(feature = "std")]
impl<T> BinaryHeapQueue<T>
where
    T: Ord,
{
    /// Create a BinaryHeapQueue with the given capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self(std::collections::BinaryHeap::with_capacity(capacity))
    }
}

#[cfg(feature = "std")]
impl<T> Default for BinaryHeapQueue<T>
where
    T: Ord,
{
    fn default() -> Self {
        Self::with_capacity(1024)
    }
}

impl<T, OrdOrder> core::convert::From<(usize, T)> for TickItem<T, OrdOrder> {
    fn from(item: (usize, T)) -> Self {
        Self {
            tick: item.0,
            item: item.1,
            _ord: PhantomData,
        }
    }
}

impl<T, OrdOrder> core::convert::Into<(usize, T)> for TickItem<T, OrdOrder> {
    fn into(self) -> (usize, T) {
        (self.tick, self.item)
    }
}

impl<T> Ord for TickItem<T, NormalOrd>
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

impl<T> Ord for TickItem<T, ReverseOrd>
where
    T: Ord,
{
    fn cmp(&self, other: &Self) -> Ordering {
        match self.tick.cmp(&other.tick) {
            Ordering::Less => Ordering::Greater,
            Ordering::Greater => Ordering::Less,
            Ordering::Equal => self.item.cmp(&other.item).reverse(),
        }
    }
}

impl<T, OrdOrder> PartialOrd for TickItem<T, OrdOrder>
where
    T: Ord,
    Self: Ord,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T, OrdOrder> PartialEq for TickItem<T, OrdOrder>
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

impl<T, OrdOrder> Eq for TickItem<T, OrdOrder> where T: Eq {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn binary_heap() {
        let mut q: BinaryHeapQueue<usize> = BinaryHeapQueue::default();
        assert!(q.enqueue(0, 12).is_ok());
        assert!(q.enqueue(0, 1).is_ok());
        assert_eq!(q.dequeue_lt(0), None);
        assert_eq!(q.dequeue_lt(1), Some((0, 1)));
        assert_eq!(q.dequeue_lt(1), Some((0, 12)));
        assert_eq!(q.dequeue_lt(1), None);
        assert_eq!(q.dequeue_lt(0), None);
        assert_eq!(q.dequeue_lt(100), None);

        assert!(q.enqueue(0, 1).is_ok());
        assert!(q.enqueue(1, 1).is_ok());
        assert!(q.enqueue(10, 0).is_ok());
        assert_eq!(q.dequeue_lt(0), None);
        assert_eq!(q.dequeue_lt(11), Some((0, 1)));
        assert_eq!(q.dequeue_lt(11), Some((1, 1)));
        assert_eq!(q.dequeue_lt(11), Some((10, 0)));
        assert_eq!(q.dequeue_lt(11), None);

        assert!(q.enqueue(20, 10000).is_ok());
        assert!(q.enqueue(22, 0).is_ok());
        assert_eq!(q.dequeue_lt(0), None);
        assert_eq!(q.dequeue_lt(20), None);
        assert_eq!(q.dequeue_lt(24), Some((20, 10000)));
        assert!(q.enqueue(2, 32).is_ok());
        assert_eq!(q.dequeue_lt(24), Some((2, 32)));
        assert_eq!(q.dequeue_lt(24), Some((22, 0)));
        assert_eq!(q.dequeue_lt(24), None);
    }
}
