use super::*;

///Get a value out of a ParamKeyValueGet, return `Default::default` if the index is out of
///range.
pub struct KeyValueGetDefault<T, I, P>
where
    T: Copy + Default,
    I: ParamGet<usize>,
    P: ParamKeyValueGet<T>,
{
    param: P,
    index: I,
    _phantom: core::marker::PhantomData<fn() -> T>,
}

pub struct KeyValueSet<T, I, P>
where
    T: Copy,
    I: ParamGet<usize>,
    P: ParamKeyValueSet<T>,
{
    param: P,
    index: I,
    _phantom: core::marker::PhantomData<fn() -> T>,
}

impl<T, I, P> KeyValueGetDefault<T, I, P>
where
    T: Copy + Default,
    I: ParamGet<usize>,
    P: ParamKeyValueGet<T>,
{
    pub fn new(param: P, index: I) -> Self {
        Self {
            param,
            index,
            _phantom: Default::default(),
        }
    }
}

impl<T, I, P> ParamGet<T> for KeyValueGetDefault<T, I, P>
where
    T: Copy + Default,
    I: ParamGet<usize>,
    P: ParamKeyValueGet<T>,
{
    fn get(&self) -> T {
        self.param.get_at(self.index.get()).unwrap_or_default()
    }
}

impl<T, I, P> KeyValueSet<T, I, P>
where
    T: Copy,
    I: ParamGet<usize>,
    P: ParamKeyValueSet<T>,
{
    pub fn new(param: P, index: I) -> Self {
        Self {
            param,
            index,
            _phantom: Default::default(),
        }
    }
}

impl<T, I, P> ParamSet<T> for KeyValueSet<T, I, P>
where
    T: Copy,
    I: ParamGet<usize>,
    P: ParamKeyValueSet<T>,
{
    fn set(&self, value: T) {
        let _ = self.param.set_at(self.index.get(), value);
    }
}
