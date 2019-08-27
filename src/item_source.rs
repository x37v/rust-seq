/// A source for items of type T that can be sent across threads
pub trait ItemSource<T, O>: Send + Sync {
    fn try_get(&mut self, init: T) -> Result<O, T>;
}
