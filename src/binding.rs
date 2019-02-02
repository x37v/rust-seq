use crate::midi::MidiValue;
use core::ops::Deref;

pub mod atomic;
pub mod bpm;
pub mod generators;
pub mod latch;
pub mod ops;
pub mod set;
pub mod spinlock;

#[cfg(feature = "std")]
pub mod cache;
#[cfg(feature = "std")]
pub mod observable;

#[cfg(feature = "alloc")]
use std::sync::Arc;

#[cfg(feature = "alloc")]
pub type BindingSetP<T> = Arc<dyn ParamBindingSet<T>>;
#[cfg(not(feature = "alloc"))]
pub type BindingSetP<T> = &'static dyn ParamBindingSet<T>;

#[cfg(feature = "alloc")]
pub type BindingLatchP<'a> = Arc<dyn ParamBindingLatch + 'a>;
#[cfg(not(feature = "alloc"))]
pub type BindingLatchP<'a> = &'static dyn ParamBindingLatch;

pub trait ParamBindingGet<T>: Send + Sync {
    fn get(&self) -> T;
}

pub trait ParamBindingSet<T>: Send + Sync {
    fn set(&self, value: T);
}

pub trait ParamBindingLatch: Send + Sync {
    fn store(&self);
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

impl<T> ParamBindingGet<T> for T
where
    T: Copy + Send + Sync,
{
    fn get(&self) -> T {
        *self
    }
}

impl<T> ParamBindingGet<T> for &T
where
    T: Copy + Sync,
{
    fn get(&self) -> T {
        **self
    }
}

#[cfg(feature = "alloc")]
impl<T> ParamBindingGet<T> for Arc<T>
where
    T: Send + Sync + ParamBindingGet<T>,
{
    fn get(&self) -> T {
        Arc::deref(self).get()
    }
}

#[cfg(feature = "alloc")]
impl<T> ParamBindingGet<T> for Arc<dyn ParamBindingGet<T>>
where
    T: Send + Sync,
{
    fn get(&self) -> T {
        Arc::deref(self).get()
    }
}

impl<T> ParamBindingGet<T> for &dyn ParamBindingGet<T>
where
    T: Send + Sync,
{
    fn get(&self) -> T {
        self.deref().get()
    }
}

#[cfg(feature = "alloc")]
impl<T> ParamBindingSet<T> for Arc<T>
where
    T: Send + Sync + ParamBindingSet<T>,
{
    fn set(&self, value: T) {
        Arc::deref(self).set(value)
    }
}

#[cfg(feature = "alloc")]
impl<T> ParamBindingSet<T> for Arc<dyn ParamBindingSet<T>>
where
    T: Send + Sync,
{
    fn set(&self, value: T) {
        Arc::deref(self).set(value)
    }
}

impl<T> ParamBindingSet<T> for &dyn ParamBindingSet<T>
where
    T: Send + Sync,
{
    fn set(&self, value: T) {
        self.deref().set(value)
    }
}
