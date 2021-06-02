//! Parameters

pub mod bool;
pub mod bpm;
pub mod ops;

//impl for atomic
mod atomic;

pub trait ParamGet<T>: Send + Sync {
    fn get(&self) -> T;
}

pub trait ParamSet<T>: Send + Sync {
    fn set(&self, value: T);
}

pub trait ParamKeyValueGet<T>: Send + Sync {
    fn get_at(&self, key: usize) -> Option<T>;
    fn len(&self) -> Option<usize>;
    //should there be an indication if its sparce? ie Array v. HashMap
}

pub trait ParamKeyValueSet<T>: Send + Sync {
    fn set_at(&self, key: usize, value: T) -> Result<(), T>;
    fn len(&self) -> Option<usize>;
    //should there be an indication if its sparce? ie Array v. HashMap
}

/// A wrapper type that implements exposing both Get and Set traits for types that impl both Get
/// and Set. So we an put this in an Arc and then cast to either
pub struct ParamGetSet<T, P>
where
    T: Copy,
    P: ParamGet<T> + ParamSet<T>,
{
    param: P,
    _phantom: core::marker::PhantomData<fn() -> T>,
}

/// A wrapper type that implements exposing both Get and Set traits for types that impl both Get
/// and Set. So we an put this in an Arc and then cast to either
pub struct ParamKeyValueGetSet<T, P>
where
    T: Copy,
    P: ParamKeyValueGet<T> + ParamKeyValueSet<T>,
{
    param: P,
    _phantom: core::marker::PhantomData<fn() -> T>,
}

impl<T, P> ParamGetSet<T, P>
where
    T: Copy,
    P: ParamGet<T> + ParamSet<T>,
{
    pub fn new(param: P) -> Self {
        Self {
            param,
            _phantom: Default::default(),
        }
    }
}

impl<T, P> ParamGet<T> for ParamGetSet<T, P>
where
    T: Copy,
    P: ParamGet<T> + ParamSet<T>,
{
    fn get(&self) -> T {
        self.param.get()
    }
}

impl<T, P> ParamSet<T> for ParamGetSet<T, P>
where
    T: Copy,
    P: ParamGet<T> + ParamSet<T>,
{
    fn set(&self, value: T) {
        self.param.set(value);
    }
}

impl<T, P> ParamKeyValueGetSet<T, P>
where
    T: Copy,
    P: ParamKeyValueGet<T> + ParamKeyValueSet<T>,
{
    pub fn new(param: P) -> Self {
        Self {
            param,
            _phantom: Default::default(),
        }
    }
}

impl<T, P> ParamKeyValueGet<T> for ParamKeyValueGetSet<T, P>
where
    T: Copy,
    P: ParamKeyValueGet<T> + ParamKeyValueSet<T>,
{
    fn get_at(&self, key: usize) -> Option<T> {
        self.param.get_at(key)
    }
    fn len(&self) -> Option<usize> {
        ParamKeyValueGet::len(&self.param)
    }
}

impl<T, P> ParamKeyValueSet<T> for ParamKeyValueGetSet<T, P>
where
    T: Copy,
    P: ParamKeyValueGet<T> + ParamKeyValueSet<T>,
{
    fn set_at(&self, key: usize, value: T) -> Result<(), T> {
        self.param.set_at(key, value)
    }
    fn len(&self) -> Option<usize> {
        ParamKeyValueSet::len(&self.param)
    }
}

impl<T> ParamGet<T> for T
where
    T: Copy + Send + Sync,
{
    fn get(&self) -> T {
        *self
    }
}

impl<T> ParamSet<T> for ()
where
    T: Copy + Sync + Send,
{
    fn set(&self, _v: T) {}
}

use crate::spin::mutex::spin::SpinMutex;

impl<T> ParamGet<T> for &'static SpinMutex<T>
where
    T: Copy + Sync + Send,
{
    fn get(&self) -> T {
        *self.lock()
    }
}

impl<T> ParamSet<T> for &'static SpinMutex<T>
where
    T: Copy + Sync + Send,
{
    fn set(&self, v: T) {
        *self.lock() = v;
    }
}

impl<T> ParamGet<T> for &'static dyn ParamGet<T>
where
    T: Copy + Sync + Send,
{
    fn get(&self) -> T {
        (*self).get()
    }
}

impl<T> ParamSet<T> for &'static dyn ParamSet<T>
where
    T: Copy + Sync + Send,
{
    fn set(&self, v: T) {
        (*self).set(v)
    }
}

impl<T> ParamKeyValueGet<T> for &'static dyn ParamKeyValueGet<T>
where
    T: Copy + Sync + Send,
{
    fn get_at(&self, index: usize) -> Option<T> {
        (*self).get_at(index)
    }

    fn len(&self) -> Option<usize> {
        (*self).len()
    }
}

impl<T> ParamKeyValueSet<T> for &'static dyn ParamKeyValueSet<T>
where
    T: Copy + Sync + Send,
{
    fn set_at(&self, key: usize, value: T) -> Result<(), T> {
        (*self).set_at(key, value)
    }

    fn len(&self) -> Option<usize> {
        (*self).len()
    }
}

impl<T, const N: usize> ParamKeyValueGet<T> for [T; N]
where
    T: Copy + Sync + Send,
{
    fn get_at(&self, index: usize) -> Option<T> {
        if index < N {
            Some(self[index])
        } else {
            None
        }
    }

    fn len(&self) -> Option<usize> {
        Some(N)
    }
}
