//! Wrappers that store the last get and or set value
//!
//! Mostly useful for wrapping generators so that we can observe what value has been used without
//! altering it.

use crate::binding::{ParamBinding, ParamBindingGet, ParamBindingSet};

/// Wrapper for a `ParamBindingGet`, caches the last get value so it can be observed later.
pub struct BindingLastGet<T, B> {
    last_value: spin::Mutex<Option<T>>,
    binding: B,
}

/// Wrapper for a `ParamBindingSet`, caches the last set value so it can be observed later.
pub struct BindingLastSet<T, B> {
    last_value: spin::Mutex<Option<T>>,
    binding: B,
}

/// Wrapper for a `ParamBinding`, caches the last get and set values so they can be observed later.
pub struct BindingLastGetSet<T, B> {
    last_get: spin::Mutex<Option<T>>,
    last_set: spin::Mutex<Option<T>>,
    binding: B,
}

impl<T, B> ParamBindingGet<T> for BindingLastGet<T, B>
where
    T: Send + Copy,
    B: ParamBindingGet<T>,
{
    fn get(&self) -> T {
        let mut g = self.last_value.lock();
        let v = self.binding.get();
        *g = Some(v);
        v
    }
}

impl<T, B> BindingLastGet<T, B>
where
    T: Send + Copy,
{
    /// Construct a BindingLastGet, wrapping the given binding.
    pub fn new(binding: B) -> Self {
        Self {
            last_value: spin::Mutex::new(None),
            binding,
        }
    }
    /// Get the last value that the binding gave, if there has been one.
    pub fn last_get(&self) -> Option<T> {
        *self.last_value.lock()
    }
}

impl<T, B> ParamBindingSet<T> for BindingLastSet<T, B>
where
    T: Send + Copy,
    B: ParamBindingSet<T>,
{
    fn set(&self, value: T) {
        let mut g = self.last_value.lock();
        self.binding.set(value);
        *g = Some(value);
    }
}

impl<T, B> BindingLastSet<T, B>
where
    T: Send + Copy,
{
    /// Construct a BindingLastSet, wrapping the given binding.
    pub fn new(binding: B) -> Self {
        Self {
            last_value: spin::Mutex::new(None),
            binding,
        }
    }
    /// Get the last value that the binding gave, if there has been one.
    pub fn last_set(&self) -> Option<T> {
        *self.last_value.lock()
    }
}

impl<T, B> ParamBindingGet<T> for BindingLastGetSet<T, B>
where
    T: Send + Copy,
    B: ParamBinding<T>,
{
    fn get(&self) -> T {
        let mut g = self.last_get.lock();
        let v = self.binding.get();
        *g = Some(v);
        v
    }
}

impl<T, B> ParamBindingSet<T> for BindingLastGetSet<T, B>
where
    T: Send + Copy,
    B: ParamBinding<T>,
{
    fn set(&self, value: T) {
        let mut g = self.last_set.lock();
        self.binding.set(value);
        *g = Some(value);
    }
}

impl<T, B> BindingLastGetSet<T, B>
where
    T: Send + Copy,
{
    /// Construct a BindingLastSet, wrapping the given binding.
    pub fn new(binding: B) -> Self {
        Self {
            last_get: spin::Mutex::new(None),
            last_set: spin::Mutex::new(None),
            binding,
        }
    }
    /// Get the last value that the binding was set to, if there has been one.
    pub fn last_set(&self) -> Option<T> {
        *self.last_set.lock()
    }
    /// Get the last value that the binding gave, if there has been one.
    pub fn last_get(&self) -> Option<T> {
        *self.last_get.lock()
    }
}
