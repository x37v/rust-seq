pub trait LinkedList<T> {
    fn len(&self) -> usize;
    fn push_back(&mut self, element: T);
    fn pop_front(&mut self) -> Option<T>;
}

pub trait ListPeekPop<T> {
    /// Get all the items in `range`, pass them and their index to `func`, if `func` returns false
    /// then remove that item from the list.
    fn peek_range_pop_if<F>(range: core::ops::Range, func: F)
    where
        F: Fn(usize, &T) -> bool;

    /// Get an item from the list by index, pass it to `func` and remove it if `func` returns
    /// false.
    fn peek_pop_if<F>(index: usize, func: F)
    where
        F: Fn(&T) -> bool;
}
