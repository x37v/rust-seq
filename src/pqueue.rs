/// A trait representing a priority queue.
pub trait PriorityQueue<N, T> {
    ///
    /// Insert `element` at `index`, returning `true` if successful.
    ///
    /// # Arguments
    /// * `index` - the index to associate with the element
    /// * `element` - the element to insert at the index given
    fn insert(&mut self, index: N, element: T) -> bool;

    ///
    /// Pop `Some(index, element)` if `func` returns true and there are elements in the queue.
    /// Returns `None` if `func` returns `false` or there are no elements left in the queue.
    ///
    /// # Arguments
    /// * `func` - the function that determines the pop conditions: takes an index and returns
    /// bool, indicating if that the element at that index should removed from the queue and
    /// returned.
    ///
    /// # Remarks
    /// Pop should be done in order.
    fn pop_if<F>(&mut self, func: F) -> Option<(N, T)>
    where
        F: Fn(&N) -> bool;
}
