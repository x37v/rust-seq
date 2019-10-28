/// A trait representing a priority queue.
pub trait PriorityQueue<N, T> {
    ///
    /// Insert `element` at `index`, returning `true` if successful.
    ///
    /// # Arguments
    /// * `index` - the index to associate with the element
    /// * `element` - the element to insert at the index given
    fn insert(&mut self, index: N, element: T) -> bool;

    /// Pop if less than the given index
    fn pop_lt(&mut self, index: N) -> Option<(N, T)>;
}
