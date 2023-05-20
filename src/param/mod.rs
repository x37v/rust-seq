//! Parameters

pub mod bool;
pub mod bpm;
//pub mod one_shot;
//pub mod ops;

//impl for atomic
mod atomic;

pub trait ParamGet<T, U> {
    fn get(&self, user_data: &mut U) -> T;
}

pub trait ParamSet<T, U> {
    fn set(&self, value: T, user_data: &mut U);
}

pub trait ParamKeyValueGet<T, U> {
    fn get_at(&self, key: usize, user_data: &mut U) -> Option<T>;
    fn len(&self, user_data: &mut U) -> Option<usize>;
    //should there be an indication if its sparce? ie Array v. HashMap
}

pub trait ParamKeyValueSet<T, U> {
    fn set_at(&self, key: usize, value: T, user_data: &mut U) -> Result<(), T>;
    fn len(&self, user_data: &mut U) -> Option<usize>;
    //should there be an indication if its sparce? ie Array v. HashMap
}

/// A wrapper type that implements exposing both Get and Set traits for types that impl both Get
/// and Set. So we an put this in an Arc and then cast to either
pub struct ParamGetSet<T, P, U>
where
    T: Copy,
    P: ParamGet<T, U> + ParamSet<T, U>,
{
    param: P,
    _phantom: core::marker::PhantomData<(T, U)>,
}

/// A wrapper type that implements exposing both Get and Set traits for types that impl both Get
/// and Set. So we an put this in an Arc and then cast to either
pub struct ParamKeyValueGetSet<T, P, U>
where
    T: Copy,
    P: ParamKeyValueGet<T, U> + ParamKeyValueSet<T, U>,
{
    param: P,
    _phantom: core::marker::PhantomData<(T, U)>,
}

impl<T, P, U> ParamGetSet<T, P, U>
where
    T: Copy,
    P: ParamGet<T, U> + ParamSet<T, U>,
{
    pub fn new(param: P) -> Self {
        Self {
            param,
            _phantom: Default::default(),
        }
    }
}

impl<T, P, U> ParamGet<T, U> for ParamGetSet<T, P, U>
where
    T: Copy,
    P: ParamGet<T, U> + ParamSet<T, U>,
{
    fn get(&self, user_data: &mut U) -> T {
        self.param.get(user_data)
    }
}

impl<T, P, U> ParamSet<T, U> for ParamGetSet<T, P, U>
where
    T: Copy,
    P: ParamGet<T, U> + ParamSet<T, U>,
{
    fn set(&self, value: T, user_data: &mut U) {
        self.param.set(value, user_data);
    }
}

impl<T, P, U> ParamKeyValueGetSet<T, P, U>
where
    T: Copy,
    P: ParamKeyValueGet<T, U> + ParamKeyValueSet<T, U>,
{
    pub fn new(param: P) -> Self {
        Self {
            param,
            _phantom: Default::default(),
        }
    }
}

impl<T, P, U> ParamKeyValueGet<T, U> for ParamKeyValueGetSet<T, P, U>
where
    T: Copy,
    P: ParamKeyValueGet<T, U> + ParamKeyValueSet<T, U>,
{
    fn get_at(&self, key: usize, user_data: &mut U) -> Option<T> {
        self.param.get_at(key, user_data)
    }
    fn len(&self, user_data: &mut U) -> Option<usize> {
        ParamKeyValueGet::len(&self.param, user_data)
    }
}

impl<T, P, U> ParamKeyValueSet<T, U> for ParamKeyValueGetSet<T, P, U>
where
    T: Copy,
    P: ParamKeyValueGet<T, U> + ParamKeyValueSet<T, U>,
{
    fn set_at(&self, key: usize, value: T, user_data: &mut U) -> Result<(), T> {
        self.param.set_at(key, value, user_data)
    }
    fn len(&self, user_data: &mut U) -> Option<usize> {
        ParamKeyValueSet::len(&self.param, user_data)
    }
}

impl<T, U> ParamGet<T, U> for T
where
    T: Copy,
{
    fn get(&self, _user_data: &mut U) -> T {
        *self
    }
}

impl<T, U> ParamSet<T, U> for ()
where
    T: Copy,
{
    fn set(&self, _v: T, _user_data: &mut U) {}
}

/* use crate::spin::mutex::spin::SpinMutex;

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

impl<T, const N: usize> ParamKeyValueGet<T> for SpinMutex<[T; N]>
where
    T: Copy + Sync + Send,
{
    fn get_at(&self, index: usize) -> Option<T> {
        if index < N {
            Some(self.lock()[index])
        } else {
            None
        }
    }

    fn len(&self) -> Option<usize> {
        Some(N)
    }
}

impl<T, const N: usize> ParamKeyValueSet<T> for SpinMutex<[T; N]>
where
    T: Copy + Sync + Send,
{
    fn set_at(&self, index: usize, value: T) -> Result<(), T> {
        if index < N {
            self.lock()[index] = value;
            Ok(())
        } else {
            Err(value)
        }
    }

    fn len(&self) -> Option<usize> {
        Some(N)
    }
} */

impl<T, U> ParamGet<T, U> for &'static dyn ParamGet<T, U>
where
    T: Copy,
{
    fn get(&self, user_data: &mut U) -> T {
        (*self).get(user_data)
    }
}

impl<T, U> ParamSet<T, U> for &'static dyn ParamSet<T, U>
where
    T: Copy,
{
    fn set(&self, v: T, user_data: &mut U) {
        (*self).set(v, user_data)
    }
}

impl<T, U> ParamKeyValueGet<T, U> for &'static dyn ParamKeyValueGet<T, U>
where
    T: Copy,
{
    fn get_at(&self, index: usize, user_data: &mut U) -> Option<T> {
        (*self).get_at(index, user_data)
    }

    fn len(&self, user_data: &mut U) -> Option<usize> {
        (*self).len(user_data)
    }
}

impl<T, U> ParamKeyValueSet<T, U> for &'static dyn ParamKeyValueSet<T, U>
where
    T: Copy,
{
    fn set_at(&self, key: usize, value: T, user_data: &mut U) -> Result<(), T> {
        (*self).set_at(key, value, user_data)
    }

    fn len(&self, user_data: &mut U) -> Option<usize> {
        (*self).len(user_data)
    }
}

impl<T, U, const N: usize> ParamKeyValueGet<T, U> for [T; N]
where
    T: Copy,
{
    fn get_at(&self, index: usize, _user_data: &mut U) -> Option<T> {
        if index < N {
            Some(self[index])
        } else {
            None
        }
    }

    fn len(&self, _user_data: &mut U) -> Option<usize> {
        Some(N)
    }
}
