use midi::MidiValue;
use ptr::ShrPtr;

pub mod atomic;
pub mod bpm;
pub mod cache;
pub mod generators;
pub mod latch;
pub mod observable;
pub mod ops;
pub mod set;
pub mod spinlock;

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
