pub trait LinkedList<T> {
    fn len(&self) -> usize;
    fn push_back(&mut self, element: T);
    fn pop_front(&mut self) -> Option<T>;
}
