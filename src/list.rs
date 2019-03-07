pub trait LinkedList<T> {
    fn len(&self) -> usize;
    fn push_back(&mut self, element: T);
    fn pop_front(&mut self) -> Option<T>;
    fn iterate(&mut self, f: Fn(&mut T) -> bool);
}
