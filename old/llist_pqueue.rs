use crate::pqueue::PriorityQueue;
cfg_if! {
    if #[cfg(feature = "std")] {
use xnor_llist::List as LList;
use xnor_llist::Node as LNode;

struct LListPQueueNode<T> {
    index: usize,
    element: Option<T>,
}

pub struct LListPQueue<T> {
    list: LList<LListPQueueNode<T>>,
}

impl<T> LListPQueue<T> {
    pub fn new() -> Self {
        Self { list: LList::new() }
    }
}

impl<T> Default for LListPQueue<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> PriorityQueue<usize, T> for LListPQueue<T> {
    fn insert(&mut self, index: usize, element: T) -> bool {
        false
    }

    fn pop_lt(&mut self, index: usize) -> Option<(usize, T)> {
        None
    }
}
}
}
