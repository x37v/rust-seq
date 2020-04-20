use super::*;
use core::marker::PhantomData;

pub struct BindingDefault<T> {
    _phantom: PhantomData<fn() -> T>,
}

impl<T> BindingDefault<T> {
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<T> ParamBindingGet<T> for BindingDefault<T>
where
    T: Default,
{
    fn get(&self) -> T {
        T::default()
    }
}

impl<T> ParamBindingSet<T> for BindingDefault<T> {
    fn set(&self, _value: T) {
        //do nothing
    }
}
