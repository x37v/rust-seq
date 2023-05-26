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
{
    pub fn new(param: P, index: I) -> Self {
        Self {
            param,
            index,
            _phantom: Default::default(),
        }
    }
}

impl<T, I, P, U> ParamGet<T, U> for KeyValueGetDefault<T, I, P>
where
    T: Copy + Default,
    I: ParamGet<usize, U>,
    P: ParamKeyValueGet<T, U>,
{
    fn get(&self, user_data: &mut U) -> T {
        self.param
            .get_at(self.index.get(user_data), user_data)
            .unwrap_or_default()
    }
}

impl<T, I, P> KeyValueSet<T, I, P> {
    pub fn new(param: P, index: I) -> Self {
        Self {
            param,
            index,
            _phantom: Default::default(),
        }
    }
}

impl<T, I, P, U> ParamSet<T, U> for KeyValueSet<T, I, P>
where
    T: Copy,
    I: ParamGet<usize, U>,
    P: ParamKeyValueSet<T, U>,
{
    fn set(&self, value: T, user_data: &mut U) {
        let _ = self
            .param
            .set_at(self.index.get(user_data), value, user_data);
    }
}

impl<I, O, F, P> GetUnaryOp<I, O, F, P>
where
    F: Fn(I) -> O,
{
    pub fn new(func: F, param: P) -> Self {
        Self {
            param,
            func,
            _phantom: PhantomData,
        }
    }
}

impl<I, O, F, P, U> ParamGet<O, U> for GetUnaryOp<I, O, F, P>
where
    F: Fn(I) -> O + Send,
    P: ParamGet<I, U>,
{
    fn get(&self, user_data: &mut U) -> O {
        (self.func)(self.param.get(user_data))
    }
}

impl<IL, IR, O, F, BL, BR> GetBinaryOp<IL, IR, O, F, BL, BR>
where
    F: Fn(IL, IR) -> O,
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

impl<IL, IR, O, F, BL, BR, U> ParamGet<O, U> for GetBinaryOp<IL, IR, O, F, BL, BR>
where
    F: Fn(IL, IR) -> O + Send,
    BL: ParamGet<IL, U>,
    BR: ParamGet<IR, U>,
{
    fn get(&self, user_data: &mut U) -> O {
        (self.func)(self.left.get(user_data), self.right.get(user_data))
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

impl<I, F, U> ParamSet<I, U> for SetUnaryOp<I, F>
where
    F: Fn(I) + Send,
{
    fn set(&self, v: I, _user_data: &mut U) {
        (self.func)(v)
    }
}

impl<IL, IR, F, P> SetBinaryOpRight<IL, IR, F, P>
where
    F: Fn(IL, IR),
{
    pub fn new(func: F, param: P) -> Self {
        Self {
            param,
            func,
            _phantom: PhantomData,
        }
    }
}

impl<IL, IR, F, P, U> ParamSet<IR, U> for SetBinaryOpRight<IL, IR, F, P>
where
    F: Fn(IL, IR) + Send,
    P: ParamGet<IL, U>,
{
    fn set(&self, v: IR, user_data: &mut U) {
        (self.func)(self.param.get(user_data), v)
    }
}

impl<IL, IR, F, P> SetBinaryOpLeft<IL, IR, F, P>
where
    F: Fn(IL, IR),
{
    pub fn new(func: F, param: P) -> Self {
        Self {
            param,
            func,
            _phantom: PhantomData,
        }
    }
}

impl<IL, IR, F, P, U> ParamSet<IL, U> for SetBinaryOpLeft<IL, IR, F, P>
where
    F: Fn(IL, IR) + Send,
    P: ParamGet<IR, U>,
{
    fn set(&self, v: IL, user_data: &mut U) {
        (self.func)(v, self.param.get(user_data))
    }
}
