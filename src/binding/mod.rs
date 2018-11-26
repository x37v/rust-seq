use midi::MidiValue;
use std::sync::Arc;

pub mod atomic;
pub mod bpm;
pub mod cache;
pub mod latch;
pub mod observable;
pub mod ops;
pub mod set;
pub mod spinlock;

pub type BindingP<T> = Arc<dyn ParamBinding<T>>;
pub type BindingGetP<T> = Arc<dyn ParamBindingGet<T>>;
pub type BindingSetP<T> = Arc<dyn ParamBindingSet<T>>;
pub type BindingLatchP<'a> = Arc<dyn ParamBindingLatch + 'a>;

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
    fn as_param_get(&self) -> &ParamBindingGet<T>;
    fn as_param_set(&self) -> &ParamBindingSet<T>;
}

impl<X, T> ParamBinding<T> for X
where
    X: ParamBindingGet<T> + ParamBindingSet<T>,
{
    fn as_param_get(&self) -> &ParamBindingGet<T> {
        self
    }
    fn as_param_set(&self) -> &ParamBindingSet<T> {
        self
    }
}

///implement get for sync types
impl<T: Copy + Send + Sync> ParamBindingGet<T> for T {
    fn get(&self) -> T {
        *self
    }
}
