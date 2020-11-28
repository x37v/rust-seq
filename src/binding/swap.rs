//! Swappable bindings.
use crate::binding::{ParamBindingGet, ParamBindingSet};
use spin::Mutex;
use std::sync::Arc;

type ABindingGet<T> = Arc<dyn ParamBindingGet<T>>;
type ABindingSet<T> = Arc<dyn ParamBindingSet<T>>;

/// A binding that can have its source swapped, gives a default value if nothing is bound
pub struct BindingSwapGet<T> {
    default: T,
    binding: Mutex<Option<ABindingGet<T>>>,
}

/// A binding that can have its destination swapped, does nothing if nothing is bound
pub struct BindingSwapSet<T> {
    binding: Mutex<Option<ABindingSet<T>>>,
}

impl<T> BindingSwapGet<T> {
    pub fn new(default: T) -> Self {
        Self {
            default,
            binding: Mutex::new(None),
        }
    }

    pub fn is_bound(&self) -> bool {
        self.binding.lock().is_some()
    }

    pub fn bind(&self, binding: ABindingGet<T>) -> Option<ABindingGet<T>> {
        self.binding.lock().replace(binding)
    }

    pub fn unbind(&self) -> Option<ABindingGet<T>> {
        self.binding.lock().take()
    }
}

impl<T> ParamBindingGet<T> for BindingSwapGet<T>
where
    T: Send + Copy + Sync,
{
    fn get(&self) -> T {
        self.binding
            .lock()
            .as_ref()
            .map_or(self.default, |b| b.get())
    }
}

impl<T> Default for BindingSwapGet<T>
where
    T: Default,
{
    fn default() -> Self {
        Self {
            default: Default::default(),
            binding: Mutex::new(None),
        }
    }
}

impl<T> BindingSwapSet<T> {
    pub fn is_bound(&self) -> bool {
        self.binding.lock().is_some()
    }

    pub fn bind(&self, binding: ABindingSet<T>) -> Option<ABindingSet<T>> {
        self.binding.lock().replace(binding)
    }

    pub fn unbind(&self) -> Option<ABindingSet<T>> {
        self.binding.lock().take()
    }
}

impl<T> Default for BindingSwapSet<T> {
    fn default() -> Self {
        Self {
            binding: Mutex::new(None),
        }
    }
}

impl<T> ParamBindingSet<T> for BindingSwapSet<T>
where
    T: Send + Copy + Sync,
{
    fn set(&self, value: T) {
        if let Some(b) = self.binding.lock().as_ref() {
            b.set(value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };

    #[test]
    fn get() {
        let g: Arc<BindingSwapGet<usize>> = Arc::new(BindingSwapGet::default());
        let b = g.clone() as Arc<dyn ParamBindingGet<usize>>;
        assert!(!g.is_bound());
        assert_eq!(b.get(), 0usize);

        let a = Arc::new(AtomicUsize::new(10));
        assert!(g
            .bind(a.clone() as Arc<dyn ParamBindingGet<usize>>)
            .is_none());
        assert!(g.is_bound());
        assert_eq!(b.get(), 10usize);
        assert_eq!(b.get(), 10usize);
        a.store(512, Ordering::SeqCst);
        assert_eq!(b.get(), 512usize);

        let a2 = Arc::new(AtomicUsize::new(40));
        assert!(g
            .bind(a2.clone() as Arc<dyn ParamBindingGet<usize>>)
            .is_some());
        assert!(g.is_bound());
        assert_eq!(b.get(), 40usize);

        //unbind, returns what it had
        assert!(g.unbind().is_some());
        assert!(!g.is_bound());
        assert_eq!(b.get(), 0usize);
        //second time does nothing
        assert!(g.unbind().is_none());
        assert!(!g.is_bound());
        assert_eq!(b.get(), 0usize);

        a2.store(7, Ordering::SeqCst);
        assert!(g
            .bind(a2.clone() as Arc<dyn ParamBindingGet<usize>>)
            .is_none());
        assert!(g.is_bound());
        assert_eq!(b.get(), 7usize);

        //setup alt default
        let g: Arc<BindingSwapGet<usize>> = Arc::new(BindingSwapGet::new(23));
        let b = g.clone() as Arc<dyn ParamBindingGet<usize>>;
        assert!(!g.is_bound());
        assert_eq!(b.get(), 23usize);
        assert!(g
            .bind(a2.clone() as Arc<dyn ParamBindingGet<usize>>)
            .is_none());
        assert!(g.is_bound());
        assert_eq!(b.get(), 7usize);
        assert!(g.unbind().is_some());
        assert!(!g.is_bound());
        assert_eq!(b.get(), 23usize);
    }

    #[test]
    fn set() {
        let a = Arc::new(AtomicUsize::new(40));
        let s: Arc<BindingSwapSet<usize>> = Arc::new(BindingSwapSet::default());
        let b = s.clone() as Arc<dyn ParamBindingSet<usize>>;

        assert!(!s.is_bound());
        assert_eq!(a.load(Ordering::Acquire), 40usize);

        b.set(234usize);
        assert_eq!(a.load(Ordering::Acquire), 40usize);
        assert!(!s.is_bound());

        assert!(s.bind(a.clone() as _).is_none());
        assert!(s.is_bound());
        b.set(24usize);
        assert_eq!(a.load(Ordering::Acquire), 24usize);

        let a2 = Arc::new(AtomicUsize::new(40));
        assert_eq!(a2.load(Ordering::Acquire), 40usize);
        assert!(s.bind(a2.clone() as _).is_some());
        b.set(6usize);
        assert_eq!(a.load(Ordering::Acquire), 24usize);
        assert_eq!(a2.load(Ordering::Acquire), 6usize);

        assert!(s.unbind().is_some());
        b.set(7usize);
        assert_eq!(a.load(Ordering::Acquire), 24usize);
        assert_eq!(a2.load(Ordering::Acquire), 6usize);

        assert!(s.unbind().is_none());
        b.set(7usize);
        assert_eq!(a.load(Ordering::Acquire), 24usize);
        assert_eq!(a2.load(Ordering::Acquire), 6usize);

        assert!(s.bind(a2.clone() as _).is_none());
        b.set(7usize);
        assert_eq!(a.load(Ordering::Acquire), 24usize);
        assert_eq!(a2.load(Ordering::Acquire), 7usize);
    }
}
