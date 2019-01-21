use crate::midi::MidiValue;
use crate::ptr::ShrPtr;

pub mod atomic;
pub mod bpm;
pub mod set;
pub mod spinlock;

cfg_if! {
    if #[cfg(feature = "std")] {
        pub mod generators;
        pub mod cache;
        pub mod ops;
        pub mod observable;
        pub mod latch;
    }
}

pub type BindingP<T> = ShrPtr<dyn ParamBinding<T>>;
pub type BindingGetP<T> = ShrPtr<dyn ParamBindingGet<T>>;
pub type BindingSetP<T> = ShrPtr<dyn ParamBindingSet<T>>;
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

///implement get for sync types
impl<T: Copy + Send + Sync> ParamBindingGet<T> for T {
    fn get(&self) -> T {
        *self
    }
}
