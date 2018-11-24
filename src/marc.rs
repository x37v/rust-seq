///wrapper around arc, spinlock mutex
pub struct MArc<T>(Arc<spinlock::Mutex<T>>);

impl<T> MArc<T>
where
    T: Copy + Send,
{
    pub fn new(value: T) -> Self {
        MArc(Arc::new(spinlock::Mutex::new(value)))
    }

    pub fn lock(&self) -> spinlock::MutexGuard<T> {
        self.0.lock()
    }

    pub fn locked<F: FnOnce(&mut T) -> R, R: Copy>(&self, func: F) -> R {
        let mut g = self.lock();
        func(&mut *g)
    }
}

impl<T> Clone for MArc<T> {
    fn clone(&self) -> Self {
        MArc(Arc::clone(&self.0))
    }
}
