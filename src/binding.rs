use crate::midi::MidiValue;
use crate::ptr::ShrPtr;
use core::ops::Deref;

pub mod atomic;
pub mod bpm;
pub mod generators;
pub mod latch;
pub mod ops;
pub mod set;
pub mod spinlock;

#[cfg(feature = "with_std")]
pub mod cache;
#[cfg(feature = "with_std")]
pub mod observable;

#[cfg(feature = "with_alloc")]
use std::sync::Arc;

pub type BindingP<T> = ShrPtr<dyn ParamBinding<T>>;
pub type BindingGetP<T> = ShrPtr<dyn ParamBindingGet<T>>;
#[cfg(feature = "with_alloc")]
pub type BindingSetP<T> = Arc<dyn ParamBindingSet<T>>;
#[cfg(not(feature = "with_alloc"))]
pub type BindingSetP<T> = &'static dyn ParamBindingSet<T>;
pub type BindingLatchP<'a> = ShrPtr<dyn ParamBindingLatch + 'a>;

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

#[cfg(feature = "with_alloc")]
impl<T> ParamBindingGet<T> for Arc<T>
where
    T: Send + Sync + ParamBindingGet<T>,
{
    fn get(&self) -> T {
        Arc::deref(self).get()
    }
}

#[cfg(feature = "with_alloc")]
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
