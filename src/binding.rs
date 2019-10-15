extern crate alloc;

/*
pub mod bpm;
pub mod generators;
pub mod ops;
pub mod spinlock;
*/

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

impl<T> ParamBindingGet<T> for T
where
    T: Copy + Send + Sync,
{
    fn get(&self) -> T {
        *self
    }
}

impl<T> ParamBindingGet<T> for spin::Mutex<T>
where
    T: ParamBindingGet<T>,
{
    fn get(&self) -> T {
        self.lock().get()
    }
}

impl<T> ParamBindingSet<T> for spin::Mutex<T>
where
    T: ParamBindingSet<T>,
{
    fn set(&self, value: T) {
        self.lock().set(value)
    }
}
