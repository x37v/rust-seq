extern crate alloc;

/// A sink for items of type T that can be sent across threads
pub trait ItemSink<T>: Send {
    fn try_put(&self, item: T) -> Result<(), T>;
}

pub trait ItemDispose<T>: Send {
    fn dispose_all(&self) -> Result<(), ()>;
}

pub trait ItemDisposeFunc<T>: Send {
    fn with_each(&self, func: &dyn Fn(T)) -> Result<(), ()>;
}

impl<T> ItemSink<T> for alloc::sync::Arc<spin::Mutex<dyn ItemSink<T>>>
where
    T: Send,
{
    fn try_put(&self, item: T) -> Result<(), T> {
        self.lock().try_put(item)
    }
}

impl<T> ItemSink<T> for &'static spin::Mutex<dyn ItemSink<T>>
where
    T: Send,
{
    fn try_put(&self, item: T) -> Result<(), T> {
        self.lock().try_put(item)
    }
}
