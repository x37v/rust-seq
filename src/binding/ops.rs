use crate::binding::ParamBindingGet;
use core::marker::PhantomData;

/// Clamp a numeric binding between [min, max], inclusive.
pub struct GetClamp<T, B, Min, Max> {
    binding: B,
    min: Min,
    max: Max,
    _phantom: PhantomData<fn() -> T>,
}

/// Clamp a numeric above a min value, inclusive.
pub struct GetClampAbove<T, B, Min> {
    binding: B,
    min: Min,
    _phantom: PhantomData<fn() -> T>,
}

/// Clamp a numeric below a max value, inclusive.
pub struct GetClampBelow<T, B, Max> {
    binding: B,
    max: Max,
    _phantom: PhantomData<fn() -> T>,
}

/// Sum two numeric bindings.
pub struct GetSum<T, L, R> {
    left: L,
    right: R,
    _phantom: PhantomData<fn() -> T>,
}

/// Multiply two numeric bindings.
pub struct GetMul<T, L, R> {
    left: L,
    right: R,
    _phantom: PhantomData<fn() -> T>,
}

/// Divide two numeric bindings.
///
///*Note*: this does protected against divide by zero but just provides `Default::default()` for `T`
/// so you probably still want to protect against it.
pub struct GetDiv<T, N, D> {
    num: N,
    den: D,
    _phantom: PhantomData<fn() -> T>,
}

/// Get the remainder from dividing (aka modulus) from two numeric bindings.
///
///*Note*: this does protected against divide by zero but just provides `Default::default()` for `T`
/// so you probably still want to protect against it.
pub struct GetRem<T, L, R> {
    left: L,
    right: R,
    _phantom: PhantomData<fn() -> T>,
}

/// Negate a signed numeric binding.
pub struct GetNegate<T, B> {
    binding: B,
    _phantom: PhantomData<fn() -> T>,
}

/// Cast one numeric binding to another.
///
/// *Note*: if the cast fails, this returns `Default::default()` of the destination value.
pub struct GetCast<I, O, B> {
    binding: B,
    _iphantom: PhantomData<fn() -> I>,
    _ophantom: PhantomData<fn() -> O>,
}

pub enum CmpOp {
    Equal,
    Greater,
    GreaterOrEqual,
    Less,
    LessOrEqual,
}

/// Compare two numeric bindings.
pub struct GetCmp<T, L, R> {
    cmp: CmpOp,
    left: L,
    right: R,
    _phantom: PhantomData<fn() -> T>,
}

impl<T, B, Min, Max> GetClamp<T, B, Min, Max>
where
    T: Send + Copy + PartialOrd,
    B: ParamBindingGet<T>,
    Min: ParamBindingGet<T>,
    Max: ParamBindingGet<T>,
{
    /// Construct a new `GetClamp`
    ///
    /// # Arguments
    ///
    /// * `binding` - the binding value to clamp
    /// * `min` - the binding for the minimum value
    /// * `max` - the binding for the maximum value
    pub fn new(binding: B, min: Min, max: Max) -> Self {
        Self {
            binding,
            min,
            max,
            _phantom: Default::default(),
        }
    }
}

impl<T, B, Min, Max> ParamBindingGet<T> for GetClamp<T, B, Min, Max>
where
    T: PartialOrd,
    B: ParamBindingGet<T>,
    Min: ParamBindingGet<T>,
    Max: ParamBindingGet<T>,
{
    fn get(&self) -> T {
        let b = self.binding.get();
        let min = self.min.get();
        let max = self.max.get();
        if b < min {
            min
        } else if b > max {
            max
        } else {
            b
        }
    }
}

impl<T, B, Min> GetClampAbove<T, B, Min>
where
    T: Send + Copy + PartialOrd,
    B: ParamBindingGet<T>,
    Min: ParamBindingGet<T>,
{
    /// Construct a new `GetClampAbove`
    ///
    /// # Arguments
    ///
    /// * `binding` - the binding value to clamp
    /// * `min` - the binding for the minimum value
    pub fn new(binding: B, min: Min) -> Self {
        Self {
            binding,
            min,
            _phantom: Default::default(),
        }
    }
}

impl<T, B, Min> ParamBindingGet<T> for GetClampAbove<T, B, Min>
where
    T: PartialOrd,
    B: ParamBindingGet<T>,
    Min: ParamBindingGet<T>,
{
    fn get(&self) -> T {
        let b = self.binding.get();
        let min = self.min.get();
        if b < min {
            min
        } else {
            b
        }
    }
}

impl<T, B, Max> GetClampBelow<T, B, Max>
where
    T: Send + Copy + PartialOrd,
    B: ParamBindingGet<T>,
    Max: ParamBindingGet<T>,
{
    /// Construct a new `GetClampBelow`
    ///
    /// # Arguments
    ///
    /// * `binding` - the binding value to clamp
    /// * `max` - the binding for the maximum value
    pub fn new(binding: B, max: Max) -> Self {
        Self {
            binding,
            max,
            _phantom: Default::default(),
        }
    }
}

impl<T, B, Max> ParamBindingGet<T> for GetClampBelow<T, B, Max>
where
    T: PartialOrd,
    B: ParamBindingGet<T>,
    Max: ParamBindingGet<T>,
{
    fn get(&self) -> T {
        let b = self.binding.get();
        let max = self.max.get();
        if b > max {
            max
        } else {
            b
        }
    }
}

impl<T, L, R> GetSum<T, L, R>
where
    T: Send,
    L: ParamBindingGet<T>,
    R: ParamBindingGet<T>,
{
    /// Construct a new `GetSum`
    ///
    /// # Arguments
    ///
    /// * `left` - the binding for left value of the sum
    /// * `right` - the binding for the right value of the sum
    pub fn new(left: L, right: R) -> Self {
        Self {
            left,
            right,
            _phantom: Default::default(),
        }
    }
}

impl<T, L, R> ParamBindingGet<T> for GetSum<T, L, R>
where
    T: core::ops::Add + num::Num,
    L: ParamBindingGet<T>,
    R: ParamBindingGet<T>,
{
    fn get(&self) -> T {
        self.left.get().add(self.right.get())
    }
}

impl<T, L, R> GetMul<T, L, R>
where
    T: Send,
    L: ParamBindingGet<T>,
    R: ParamBindingGet<T>,
{
    /// Construct a new `GetMul`
    ///
    /// # Arguments
    ///
    /// * `left` - the binding for left value of the multiplication
    /// * `right` - the binding for the right value of the multiplication
    pub fn new(left: L, right: R) -> Self {
        Self {
            left,
            right,
            _phantom: Default::default(),
        }
    }
}

impl<T, L, R> ParamBindingGet<T> for GetMul<T, L, R>
where
    T: core::ops::Mul + num::Num,
    L: ParamBindingGet<T>,
    R: ParamBindingGet<T>,
{
    fn get(&self) -> T {
        self.left.get().mul(self.right.get())
    }
}

impl<T, N, D> GetDiv<T, N, D>
where
    T: Send,
    N: ParamBindingGet<T>,
    D: ParamBindingGet<T>,
{
    /// Construct a new `GetDiv`
    ///
    /// # Arguments
    ///
    /// * `num` - the binding for numerator value of the division
    /// * `den` - the binding for denominator value of the division
    pub fn new(num: N, den: D) -> Self {
        Self {
            num,
            den,
            _phantom: Default::default(),
        }
    }
}

impl<T, N, D> ParamBindingGet<T> for GetDiv<T, N, D>
where
    T: core::ops::Div + num::Num + num::Zero + Default,
    N: ParamBindingGet<T>,
    D: ParamBindingGet<T>,
{
    fn get(&self) -> T {
        let d = self.den.get();
        if d.is_zero() {
            Default::default()
        } else {
            self.num.get().div(d)
        }
    }
}

impl<T, L, R> GetRem<T, L, R>
where
    T: Send,
    L: ParamBindingGet<T>,
    R: ParamBindingGet<T>,
{
    /// Construct a new `GetRem`
    ///
    /// # Arguments
    ///
    /// * `left` - the binding for left value of the division
    /// * `right` - the binding for the right value of the division
    pub fn new(left: L, right: R) -> Self {
        Self {
            left,
            right,
            _phantom: Default::default(),
        }
    }
}

impl<T, L, R> ParamBindingGet<T> for GetRem<T, L, R>
where
    T: core::ops::Rem + num::Num + num::Zero + Default,
    L: ParamBindingGet<T>,
    R: ParamBindingGet<T>,
{
    fn get(&self) -> T {
        let right = self.right.get();
        if right.is_zero() {
            Default::default()
        } else {
            self.left.get().rem(right)
        }
    }
}

impl<T, B> GetNegate<T, B>
where
    T: Send,
    B: ParamBindingGet<T>,
{
    /// Construct a new `GetNegate`
    ///
    /// # Arguments
    ///
    /// * `binding` - the binding to negate
    pub fn new(binding: B) -> Self {
        Self {
            binding,
            _phantom: Default::default(),
        }
    }
}

impl<T, B> ParamBindingGet<T> for GetNegate<T, B>
where
    T: num::Signed,
    B: ParamBindingGet<T>,
{
    fn get(&self) -> T {
        -self.binding.get()
    }
}

impl<I, O, B> GetCast<I, O, B>
where
    I: Send,
    O: Send,
    B: ParamBindingGet<I>,
{
    /// Construct a new `GetCast`
    ///
    /// # Arguments
    ///
    /// * `binding` - the binding to cast
    ///
    /// # Example
    ///
    /// Sometimes you might have to specify the destination type of the cast.
    /// Here we specify both the source and the destination, `f32` into `u8`.
    /// The type of the source binding can be discovered easily by the compiler.
    ///
    /// ```
    /// use sched::binding::ParamBindingGet;
    /// use sched::binding::ops::GetCast;
    /// use sched::ptr::ShrPtr;
    ///
    /// let f: f32 = 23f32.into();
    /// let c : ShrPtr<GetCast<f32, u8, _>> = GetCast::new(f.clone()).into();
    /// assert_eq!(23f32, f.get());
    /// assert_eq!(23u8, c.get());
    /// ```
    pub fn new(binding: B) -> Self {
        Self {
            binding,
            _iphantom: Default::default(),
            _ophantom: Default::default(),
        }
    }
}

impl<I, O, B> ParamBindingGet<O> for GetCast<I, O, B>
where
    I: num::NumCast,
    O: num::NumCast + Default,
    B: ParamBindingGet<I>,
{
    fn get(&self) -> O {
        if let Some(v) = O::from(self.binding.get()) {
            v
        } else {
            Default::default()
        }
    }
}

impl<T, L, R> GetCmp<T, L, R>
where
    T: Send,
    L: ParamBindingGet<T>,
    R: ParamBindingGet<T>,
{
    /// Construct a new `GetCmp`
    ///
    /// # Arguments
    ///
    /// * `cmp` - the comparison to execute
    /// * `left` - the binding for left value of the comparison
    /// * `right` - the binding for the right value of the comparison
    pub fn new(cmp: CmpOp, left: L, right: R) -> Self {
        Self {
            cmp,
            left,
            right,
            _phantom: Default::default(),
        }
    }
}

impl<T, L, R> ParamBindingGet<bool> for GetCmp<T, L, R>
where
    T: PartialOrd + PartialEq,
    L: ParamBindingGet<T>,
    R: ParamBindingGet<T>,
{
    fn get(&self) -> bool {
        let left = self.left.get();
        let right = self.right.get();
        match self.cmp {
            CmpOp::Equal => left.eq(&right),
            CmpOp::Greater => left.gt(&right),
            CmpOp::GreaterOrEqual => left.ge(&right),
            CmpOp::Less => left.lt(&right),
            CmpOp::LessOrEqual => left.le(&right),
        }
    }
}

#[cfg(feature = "with_std")]
mod tests {
    use super::*;
    use crate::binding::ParamBindingGet;
    use std::sync::Arc;

    #[test]
    fn clamp() {
        let min: i32 = 20;
        let max: i32 = 23;
        let mut v: i32 = 1;
        let c = GetClamp::new(v, min, max);
        assert_eq!(min, c.get());

        v = 234;
        let c = GetClamp::new(v, min, max);
        assert_eq!(max, c.get());

        v = 22;
        let c = GetClamp::new(v, min, max);
        assert_eq!(v, c.get());

        let min = Arc::new(-23);
        let max = &43;
        let v = &24;

        let c = GetClamp::new(v, min, max);
        assert_eq!(*v, c.get());

        let c2 = GetClamp::new(c, &34, &49);
        assert_eq!(34, c2.get());

        let c3: Arc<dyn ParamBindingGet<i32>> = Arc::new(GetClamp::new(30, 0, 100));
        let c4 = Arc::new(GetClamp::new(c3.clone(), -100, 100));
        assert_eq!(30, c4.get());

        //incorrect input but we want to keep going, will return max
        let c = GetClamp::new(1000isize, 100isize, -100isize);
        assert_eq!(-100isize, c.get());
    }
}
