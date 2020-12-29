//! Wrappers that store the last get and or set value
//!
//! Mostly useful for wrapping generators so that we can observe what value has been used without
//! altering it.

use crate::binding::{ParamBinding, ParamBindingGet, ParamBindingSet};

/// Trait for cached get and or set values.
pub trait BindingLast<T>: Send + Sync {
    /// Get the last value that the binding gave, if there has been one.
    fn last_get(&self) -> Option<T> {
        None
    }

    /// Get the last value that the binding was set to, if there has been one.
    fn last_set(&self) -> Option<T> {
        None
    }
}

/// Wrapper for a `ParamBindingGet`, caches the last get value so it can be observed later.
pub struct BindingLastGet<T> {
    last_value: spin::Mutex<Option<T>>,
    binding: Box<dyn ParamBindingGet<T>>,
}

/// Wrapper for a `ParamBindingSet`, caches the last set value so it can be observed later.
pub struct BindingLastSet<T> {
    last_value: spin::Mutex<Option<T>>,
    binding: Box<dyn ParamBindingSet<T>>,
}

/// Wrapper for a `ParamBinding` (Get + Set), caches the last get and set values so they can be observed later.
pub struct BindingLastGetSet<T> {
    last_get: spin::Mutex<Option<T>>,
    last_set: spin::Mutex<Option<T>>,
    binding: Box<dyn ParamBinding<T>>,
}

impl<T> ParamBindingGet<T> for BindingLastGet<T>
where
    T: Send + Copy,
{
    fn get(&self) -> T {
        let mut g = self.last_value.lock();
        let v = self.binding.get();
        *g = Some(v);
        v
    }
}

impl<T> BindingLastGet<T>
where
    T: Send + Copy,
{
    /// Construct a BindingLastGet, wrapping the given binding.
    pub fn new<B: ParamBindingGet<T> + 'static>(binding: B) -> Self {
        Self {
            last_value: spin::Mutex::new(None),
            binding: Box::new(binding),
        }
    }

    /// Construct a BindingLastGet, wrapping the given binding, initialize the last_value.
    pub fn new_init<B: ParamBindingGet<T> + 'static>(binding: B) -> Self {
        let b = Self::new(binding);
        let _ = b.get();
        b
    }
}

impl<T> BindingLast<T> for BindingLastGet<T>
where
    T: Send + Copy,
{
    /// Get the last value that the binding gave, if there has been one.
    fn last_get(&self) -> Option<T> {
        *self.last_value.lock()
    }
}

impl<T> ParamBindingSet<T> for BindingLastSet<T>
where
    T: Send + Copy,
{
    fn set(&self, value: T) {
        let mut g = self.last_value.lock();
        self.binding.set(value);
        *g = Some(value);
    }
}

impl<T> BindingLastSet<T> {
    /// Construct a BindingLastSet, wrapping the given binding.
    pub fn new<B: ParamBindingSet<T> + 'static>(binding: B) -> Self {
        Self {
            last_value: spin::Mutex::new(None),
            binding: Box::new(binding),
        }
    }
}

impl<T> BindingLast<T> for BindingLastSet<T>
where
    T: Send + Copy,
{
    /// Get the last value that the binding gave, if there has been one.
    fn last_set(&self) -> Option<T> {
        *self.last_value.lock()
    }
}

impl<T> ParamBindingGet<T> for BindingLastGetSet<T>
where
    T: Send + Copy,
{
    fn get(&self) -> T {
        let mut g = self.last_get.lock();
        let v = self.binding.get();
        *g = Some(v);
        v
    }
}

impl<T> ParamBindingSet<T> for BindingLastGetSet<T>
where
    T: Send + Copy,
{
    fn set(&self, value: T) {
        let mut g = self.last_set.lock();
        self.binding.set(value);
        *g = Some(value);
    }
}

impl<T> BindingLastGetSet<T>
where
    T: Send + Copy,
{
    /// Construct a BindingLastSet, wrapping the given binding.
    pub fn new<B: ParamBinding<T> + 'static>(binding: B) -> Self {
        Self {
            last_get: spin::Mutex::new(None),
            last_set: spin::Mutex::new(None),
            binding: Box::new(binding),
        }
    }
}

impl<T> BindingLast<T> for BindingLastGetSet<T>
where
    T: Send + Copy,
{
    /// Get the last value that the binding gave, if there has been one.
    fn last_get(&self) -> Option<T> {
        *self.last_get.lock()
    }
    /// Get the last value that the binding was set to, if there has been one.
    fn last_set(&self) -> Option<T> {
        *self.last_set.lock()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binding::{ParamBindingGet, ParamBindingSet};
    use std::sync::{atomic::AtomicUsize, Arc};

    #[test]
    fn can_wrap_get() {
        let x = Arc::new(AtomicUsize::new(0));
        let w: Arc<BindingLastGet<usize>> = Arc::new(BindingLastGet::new(
            x.clone() as Arc<dyn ParamBindingGet<usize>>
        ));
        assert_eq!(w.last_get(), None);
        assert_eq!(w.get(), 0usize);
        assert_eq!(w.last_get(), Some(0));
        assert_eq!(w.last_get(), Some(0));
        assert_eq!(w.get(), 0usize);
        assert_eq!(w.last_get(), Some(0));

        x.store(2084, std::sync::atomic::Ordering::SeqCst);
        assert_eq!(w.last_get(), Some(0));
        assert_eq!(w.get(), 2084usize);
        assert_eq!(w.last_get(), Some(2084));
    }

    #[test]
    fn can_wrap_set() {
        let x = Arc::new(AtomicUsize::new(0));
        let w: Arc<BindingLastSet<usize>> = Arc::new(BindingLastSet::new(
            x.clone() as Arc<dyn ParamBindingSet<usize>>
        ));
        assert_eq!(w.last_set(), None);
        w.set(0usize);

        assert_eq!(w.last_set(), Some(0));
        assert_eq!(w.last_set(), Some(0));

        w.set(1usize);
        assert_eq!(w.last_set(), Some(1));

        //setting the atomic directly doesn't actually work
        x.store(2084, std::sync::atomic::Ordering::SeqCst);
        assert_eq!(w.last_set(), Some(1));

        //but using the set interface does
        w.set(42usize);
        assert_eq!(w.last_set(), Some(42));
    }

    #[test]
    fn can_wrap_get_set() {
        let w: Arc<BindingLastGetSet<usize>> =
            Arc::new(BindingLastGetSet::new(AtomicUsize::new(0)));
        let s = w.clone() as Arc<dyn ParamBindingSet<usize>>;
        let g = w.clone() as Arc<dyn ParamBindingGet<usize>>;

        assert_eq!(w.last_get(), None);
        assert_eq!(w.last_set(), None);

        s.set(2084usize);
        assert_eq!(w.last_get(), None);
        assert_eq!(w.last_set(), Some(2084usize));

        assert_eq!(g.get(), 2084usize);
        assert_eq!(w.last_get(), Some(2084usize));
        assert_eq!(w.last_set(), Some(2084usize));

        assert_eq!(g.get(), 2084usize);
        assert_eq!(w.last_get(), Some(2084usize));
        assert_eq!(w.last_set(), Some(2084usize));

        s.set(42usize);
        assert_eq!(w.last_set(), Some(42usize));
        assert_eq!(w.last_get(), Some(2084usize));
        assert_eq!(g.get(), 42usize);
        assert_eq!(w.last_set(), Some(42usize));
        assert_eq!(w.last_get(), Some(42usize));
    }
}
