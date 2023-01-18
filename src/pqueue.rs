//! Priority Queues
use core::cmp::Ordering;

pub trait TickPriorityEnqueue<T> {
    /// Try to enqueue the item at the given tick
    fn try_enqueue(&mut self, tick: usize, value: T) -> Result<(), T>;
}

pub trait TickPriorityDequeue<T> {
    /// Dequeue items, in order, with a tick less than `tick`, if there are any.
    fn dequeue_lt(&mut self, tick: usize) -> Option<(usize, T)>;
}

/// markers for ordering, work the fact that BinaryHeap is a max heap, so we reverse Ord for
/// TickItem when using it.
pub struct NormalOrd;
pub struct ReverseOrd;

pub struct TickItem<T, OrdOrder = NormalOrd> {
    tick: usize,
    item: T,
    _ord: core::marker::PhantomData<OrdOrder>,
}

#[cfg(feature = "std")]
pub mod binaryheap {
    use super::*;
    pub struct BinaryHeapQueue<T>(std::collections::BinaryHeap<TickItem<T, ReverseOrd>>);

    impl<T> TickPriorityEnqueue<T> for BinaryHeapQueue<T>
    where
        T: Ord,
    {
        fn try_enqueue(&mut self, tick: usize, value: T) -> Result<(), T> {
            //don't allocate
            if self.0.len() >= self.0.capacity() {
                Err(value)
            } else {
                self.0.push((tick, value).into());
                Ok(())
            }
        }
    }

    impl<T> TickPriorityDequeue<T> for BinaryHeapQueue<T>
    where
        T: Ord,
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

    impl<T> BinaryHeapQueue<T>
    where
        T: Ord,
    {
        /// Create a BinaryHeapQueue with the given capacity
        pub fn with_capacity(capacity: usize) -> Self {
            Self(std::collections::BinaryHeap::with_capacity(capacity))
        }
    }

    impl<T> Default for BinaryHeapQueue<T>
    where
        T: Ord,
    {
        fn default() -> Self {
            Self::with_capacity(1024)
        }
    }
}

impl<T, OrdOrder> TickItem<T, OrdOrder> {
    pub fn tick(&self) -> usize {
        self.tick
    }
}

impl<T, OrdOrder> core::convert::From<(usize, T)> for TickItem<T, OrdOrder> {
    fn from(item: (usize, T)) -> Self {
        Self {
            tick: item.0,
            item: item.1,
            _ord: Default::default(),
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
        self.tick.eq(&other.tick) && self.item.eq(&other.item)
    }
}

impl<T, OrdOrder> Eq for TickItem<T, OrdOrder> where T: Eq {}
