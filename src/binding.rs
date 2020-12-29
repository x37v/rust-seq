extern crate alloc;

pub mod bpm;
pub mod generators;
pub mod hysteresis;
pub mod last;
pub mod ops;
pub mod spinlock;
pub mod swap;

use core::ops::Deref;

// include automatic impls
mod atomic;

pub trait ParamBindingGet<T>: Send + Sync {
    fn get(&self) -> T;
}

pub trait ParamBindingSet<T>: Send + Sync {
    fn set(&self, value: T);
}

pub trait ParamBinding<T>: ParamBindingSet<T> + ParamBindingGet<T> {
    fn as_param_get(&self) -> &dyn ParamBindingGet<T>;
    fn as_param_set(&self) -> &dyn ParamBindingSet<T>;
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
