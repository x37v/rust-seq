//! Priority Queues
use core::cmp::Ordering;

pub trait TickPriorityEnqueue<T>: Send {
    /// Try to enqueue the item at the given tick
    fn try_enqueue(&self, tick: usize, value: T) -> Result<(), T>;
}

pub trait TickPriorityDequeue<T>: Send {
    /// Dequeue items, in order, with a tick less than `tick`, if there are any.
    fn dequeue_lt(&self, tick: usize) -> Option<(usize, T)>;
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
    pub struct BinaryHeapQueue<T> {
        inner: std::sync::Mutex<std::collections::BinaryHeap<TickItem<T, ReverseOrd>>>,
    }

    impl<T> TickPriorityEnqueue<T> for BinaryHeapQueue<T>
    where
        T: Send + Ord,
    {
        fn try_enqueue(&self, tick: usize, value: T) -> Result<(), T> {
            let mut g = self.inner.lock().unwrap();
            //don't allocate
            if g.len() >= g.capacity() {
                Err(value)
            } else {
                g.push((tick, value).into());
                Ok(())
            }
        }
    }

    impl<T> TickPriorityDequeue<T> for BinaryHeapQueue<T>
    where
        T: Send + Ord,
    {
        fn dequeue_lt(&self, tick: usize) -> Option<(usize, T)> {
            let mut g = self.inner.lock().unwrap();
            if let Some(t) = g.peek() {
                if t.tick() < tick {
                    g.pop().map(|v| v.into())
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
            Self {
                inner: std::sync::Mutex::new(std::collections::BinaryHeap::with_capacity(capacity)),
            }
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
