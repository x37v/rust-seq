extern crate alloc;

/// A source for items of type T that can be sent across threads
pub trait ItemSource<T, O = Box<T>>: Send {
    fn try_get(&self, init: T) -> Result<O, T>;
}

impl<T, O> ItemSource<T, O> for alloc::sync::Arc<spin::Mutex<dyn ItemSource<T, O>>> {
    fn try_get(&self, init: T) -> Result<O, T> {
        self.lock().try_get(init)
    }
}

impl<T, O> ItemSource<T, O> for &'static spin::Mutex<dyn ItemSource<T, O>> {
    fn try_get(&self, init: T) -> Result<O, T> {
        self.lock().try_get(init)
    }
}
