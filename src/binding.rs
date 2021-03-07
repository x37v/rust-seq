extern crate alloc;

pub mod bpm;
pub mod generators;
pub mod hysteresis;
pub mod last;
pub mod ops;
pub mod spinlock;
pub mod swap;

use core::ops::Deref;
use std::marker::PhantomData;

// include automatic impls
mod atomic;

pub trait ParamBindingGet<T>: Send + Sync {
    fn get(&self) -> T;
}

pub trait ParamBindingSet<T>: Send + Sync {
    fn set(&self, value: T);
}

pub trait ParamBindingKeyValueGet<T>: Send + Sync {
    fn get_at(&self, key: usize) -> Option<T>;
    fn len(&self) -> Option<usize>;
    //should there be an indication if its sparce? ie Array v. HashMap
}

pub trait ParamBindingKeyValueSet<T>: Send + Sync {
    fn set_at(&self, key: usize, value: T) -> Result<(), T>;
    fn len(&self) -> Option<usize>;
    //should there be an indication if its sparce? ie Array v. HashMap
}

/// A wrapper type that implements exposing both Get and Set traits for types that impl both Get
/// and Set. So we an put this in an Arc and then cast to either
pub struct ParamBindingGetSet<T, U>
where
    T: Copy,
    U: ParamBindingGet<T> + ParamBindingSet<T>,
{
    binding: U,
    _phantom: PhantomData<fn() -> T>,
}

/// A wrapper type that implements exposing both Get and Set traits for types that impl both Get
/// and Set. So we an put this in an Arc and then cast to either
pub struct ParamBindingKeyValueGetSet<T, U>
where
    T: Copy,
    U: ParamBindingKeyValueGet<T> + ParamBindingKeyValueSet<T>,
{
    binding: U,
    _phantom: PhantomData<fn() -> T>,
}

pub trait ParamBinding<T>: ParamBindingSet<T> + ParamBindingGet<T> {
    fn as_param_get(&self) -> &dyn ParamBindingGet<T>;
    fn as_param_set(&self) -> &dyn ParamBindingSet<T>;
}

pub trait ParamBindingKeyValue<T>: ParamBindingKeyValueGet<T> + ParamBindingKeyValueSet<T> {
    fn as_param_kv_get(&self) -> &dyn ParamBindingKeyValueGet<T>;
    fn as_param_kv_set(&self) -> &dyn ParamBindingKeyValueSet<T>;
}

impl<T, U> ParamBindingGetSet<T, U>
where
    T: Copy,
    U: ParamBindingGet<T> + ParamBindingSet<T>,
{
    pub fn new(binding: U) -> Self {
        Self {
            binding,
            _phantom: Default::default(),
        }
    }
}

impl<T, U> ParamBindingGet<T> for ParamBindingGetSet<T, U>
where
    T: Copy,
    U: ParamBindingGet<T> + ParamBindingSet<T>,
{
    fn get(&self) -> T {
        self.binding.get()
    }
}

impl<T, U> ParamBindingSet<T> for ParamBindingGetSet<T, U>
where
    T: Copy,
    U: ParamBindingGet<T> + ParamBindingSet<T>,
{
    fn set(&self, value: T) {
        self.binding.set(value);
    }
}

impl<T, U> ParamBindingKeyValueGetSet<T, U>
where
    T: Copy,
    U: ParamBindingKeyValueGet<T> + ParamBindingKeyValueSet<T>,
{
    pub fn new(binding: U) -> Self {
        Self {
            binding,
            _phantom: Default::default(),
        }
    }
}

impl<T, U> ParamBindingKeyValueGet<T> for ParamBindingKeyValueGetSet<T, U>
where
    T: Copy,
    U: ParamBindingKeyValueGet<T> + ParamBindingKeyValueSet<T>,
{
    fn get_at(&self, key: usize) -> Option<T> {
        self.binding.get_at(key)
    }
    fn len(&self) -> Option<usize> {
        ParamBindingKeyValueGet::len(&self.binding)
    }
}

impl<T, U> ParamBindingKeyValueSet<T> for ParamBindingKeyValueGetSet<T, U>
where
    T: Copy,
    U: ParamBindingKeyValueGet<T> + ParamBindingKeyValueSet<T>,
{
    fn set_at(&self, key: usize, value: T) -> Result<(), T> {
        self.binding.set_at(key, value)
    }
    fn len(&self) -> Option<usize> {
        ParamBindingKeyValueSet::len(&self.binding)
    }
}

impl<X, T> ParamBinding<T> for X
where
    X: ParamBindingGet<T> + ParamBindingSet<T>,
{
    fn as_param_get(&self) -> &dyn ParamBindingGet<T> {
        self
    }
    fn as_param_set(&self) -> &dyn ParamBindingSet<T> {
        self
    }
}

impl<X, T> ParamBindingKeyValue<T> for X
where
    X: ParamBindingKeyValueGet<T> + ParamBindingKeyValueSet<T>,
{
    fn as_param_kv_get(&self) -> &dyn ParamBindingKeyValueGet<T> {
        self
    }
    fn as_param_kv_set(&self) -> &dyn ParamBindingKeyValueSet<T> {
        self
    }
}

/*
impl<U, T> ParamBindingGet<T> for U
where
    U: Send + Deref<Target = T>,
    T: Copy + Send,
{
    fn get(&self) -> T {
        *self.deref()
    }
}
*/

impl<T> ParamBindingGet<T> for T
where
    T: Copy + Send + Sync,
{
    fn get(&self) -> T {
        *self
    }
}

impl<T> ParamBindingGet<T> for &'static T
where
    T: Copy + Send + Sync,
{
    fn get(&self) -> T {
        **self
    }
}

impl<T> ParamBindingGet<T> for alloc::sync::Arc<T>
where
    T: Copy + Send + Sync,
{
    fn get(&self) -> T {
        *self.deref()
    }
}

impl<T> ParamBindingGet<T> for alloc::sync::Arc<dyn ParamBindingGet<T>>
where
    T: Copy + Send,
{
    fn get(&self) -> T {
        self.deref().get()
    }
}

impl<T> ParamBindingGet<T> for alloc::sync::Arc<dyn ParamBinding<T>>
where
    T: Copy + Send,
{
    fn get(&self) -> T {
        self.deref().get()
    }
}

impl<T> ParamBindingGet<T> for alloc::sync::Arc<spin::Mutex<dyn ParamBindingGet<T>>>
where
    T: Copy + Send,
{
    fn get(&self) -> T {
        self.lock().get()
    }
}

impl<T> ParamBindingSet<T> for ()
where
    T: Copy + Send,
{
    fn set(&self, _value: T) {}
}

impl<T> ParamBindingSet<T> for alloc::sync::Arc<dyn ParamBindingSet<T>>
where
    T: Copy + Send,
{
    fn set(&self, value: T) {
        self.deref().set(value)
    }
}

impl<T> ParamBindingSet<T> for alloc::sync::Arc<dyn ParamBinding<T>>
where
    T: Copy + Send,
{
    fn set(&self, value: T) {
        self.deref().set(value)
    }
}

impl<T> ParamBindingSet<T> for alloc::sync::Arc<spin::Mutex<dyn ParamBindingSet<T>>>
where
    T: Copy + Send,
{
    fn set(&self, value: T) {
        self.lock().set(value)
    }
}

impl<T> ParamBindingKeyValueGet<T> for alloc::sync::Arc<dyn ParamBindingKeyValue<T>>
where
    T: Copy + Send,
{
    fn get_at(&self, key: usize) -> Option<T> {
        self.as_param_kv_get().get_at(key)
    }
    fn len(&self) -> Option<usize> {
        self.as_param_kv_get().len()
    }
}

impl<T> ParamBindingKeyValueSet<T> for alloc::sync::Arc<dyn ParamBindingKeyValue<T>>
where
    T: Copy + Send,
{
    fn set_at(&self, key: usize, value: T) -> Result<(), T> {
        self.as_param_kv_set().set_at(key, value)
    }
    fn len(&self) -> Option<usize> {
        self.as_param_kv_set().len()
    }
}
