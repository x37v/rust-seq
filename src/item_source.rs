/// A source for items of type T that can be sent across threads
pub trait ItemSource<T>: Send + Sync {
    fn try_pop(&mut self) -> Result<T, core::fmt::Error>;
}
