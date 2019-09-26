/// A sink for items of type T that can be sent across threads
pub trait ItemSink<T>: Send + Sync {
    fn try_put(&mut self, item: T) -> Result<(), T>;
}
