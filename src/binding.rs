use core::ops::Deref;

mod atomic;

pub trait ParamBindingGet<T>: Send {
    fn get(&self) -> T;
}

pub trait ParamBindingSet<T>: Send {
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

impl<U, T> ParamBindingGet<T> for U
where
    U: Send + Deref<Target = T>,
    T: Copy + Send,
{
    fn get(&self) -> T {
        *self.deref()
    }
}
