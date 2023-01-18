use super::*;
use core::marker::PhantomData;

/// Helper funcs
pub mod funcs {
    /// cast or return O::default() if cast fails
    pub fn cast_or_default<I, O>(i: I) -> O
    where
        I: num_traits::NumCast,
        O: num_traits::NumCast + Default,
    {
        O::from(i).unwrap_or_default()
    }

    /// if denominator equals zero, return default, otherwise return div
    pub fn div_protected<T>(num: T, den: T) -> T
    where
        T: core::ops::Div + num_traits::Num + num_traits::Zero + Default,
    {
        if den.is_zero() {
            Default::default()
        } else {
            num.div(den)
        }
    }

    /// if denominator equals zero, return default, otherwise return remainder
    pub fn rem_protected<T>(num: T, den: T) -> T
    where
        T: core::ops::Rem + num_traits::Num + num_traits::Zero + Default,
    {
        if den.is_zero() {
            Default::default()
        } else {
            num.rem(den)
        }
    }
}

///Get a value out of a ParamKeyValueGet, return `Default::default` if the index is out of
///range.
pub struct KeyValueGetDefault<T, I, P> {
    param: P,
    index: I,
    _phantom: PhantomData<fn() -> T>,
}

pub struct KeyValueSet<T, I, P> {
    param: P,
    index: I,
    _phantom: PhantomData<fn() -> T>,
}

pub struct GetUnaryOp<I, O, F, P> {
    param: P,
    func: F,
    _phantom: PhantomData<fn() -> (I, O)>,
}

pub struct GetBinaryOp<IL, IR, O, F, PL, PR> {
    left: PL,
    right: PR,
    func: F,
    _phantom: PhantomData<fn() -> (IL, IR, O)>,
}

pub struct SetUnaryOp<I, F> {
    func: F,
    _phantom: PhantomData<fn() -> I>,
}

pub struct SetBinaryOpRight<IL, IR, F, P> {
    param: P,
    func: F,
    _phantom: PhantomData<fn() -> (IL, IR)>,
}

pub struct SetBinaryOpLeft<IL, IR, F, P> {
    param: P,
    func: F,
    _phantom: PhantomData<fn() -> (IL, IR)>,
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

impl<I, O, F, P> GetUnaryOp<I, O, F, P>
where
    F: Fn(I) -> O,
    P: ParamGet<I>,
{
    pub fn new(func: F, param: P) -> Self {
        Self {
            param,
            func,
            _phantom: PhantomData,
        }
    }
}

impl<I, O, F, P> ParamGet<O> for GetUnaryOp<I, O, F, P>
where
    F: Fn(I) -> O,
    P: ParamGet<I>,
{
    fn get(&self) -> O {
        (self.func)(self.param.get())
    }
}

impl<IL, IR, O, F, BL, BR> GetBinaryOp<IL, IR, O, F, BL, BR>
where
    F: Fn(IL, IR) -> O,
    BL: ParamGet<IL>,
    BR: ParamGet<IR>,
{
    pub fn new(func: F, left: BL, right: BR) -> Self {
        Self {
            left,
            right,
            func,
            _phantom: PhantomData,
        }
    }
}

impl<IL, IR, O, F, BL, BR> ParamGet<O> for GetBinaryOp<IL, IR, O, F, BL, BR>
where
    F: Fn(IL, IR) -> O,
    BL: ParamGet<IL>,
    BR: ParamGet<IR>,
{
    fn get(&self) -> O {
        (self.func)(self.left.get(), self.right.get())
    }
}

impl<I, F> SetUnaryOp<I, F>
where
    F: Fn(I),
{
    pub fn new(func: F) -> Self {
        Self {
            func,
            _phantom: PhantomData,
        }
    }
}

impl<I, F> ParamSet<I> for SetUnaryOp<I, F>
where
    F: Fn(I),
{
    fn set(&self, v: I) {
        (self.func)(v)
    }
}

impl<IL, IR, F, P> SetBinaryOpRight<IL, IR, F, P>
where
    F: Fn(IL, IR),
    P: ParamGet<IL>,
{
    pub fn new(func: F, param: P) -> Self {
        Self {
            param,
            func,
            _phantom: PhantomData,
        }
    }
}

impl<IL, IR, F, P> ParamSet<IR> for SetBinaryOpRight<IL, IR, F, P>
where
    F: Fn(IL, IR),
    P: ParamGet<IL>,
{
    fn set(&self, v: IR) {
        (self.func)(self.param.get(), v)
    }
}

impl<IL, IR, F, P> SetBinaryOpLeft<IL, IR, F, P>
where
    F: Fn(IL, IR),
    P: ParamGet<IR>,
{
    pub fn new(func: F, param: P) -> Self {
        Self {
            param,
            func,
            _phantom: PhantomData,
        }
    }
}

impl<IL, IR, F, P> ParamSet<IL> for SetBinaryOpLeft<IL, IR, F, P>
where
    F: Fn(IL, IR),
    P: ParamGet<IR>,
{
    fn set(&self, v: IL) {
        (self.func)(v, self.param.get())
    }
}
