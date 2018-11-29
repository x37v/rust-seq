use binding::ParamBindingGet;
use rand::prelude::*;
use std::marker::PhantomData;
use std::sync::Arc;

/// Clamp a numeric binding between [min, max], inclusive.
pub struct GetClamp<T, B, Min, Max> {
    binding: Arc<B>,
    min: Arc<Min>,
    max: Arc<Max>,
    _phantom: PhantomData<fn() -> T>,
}

/// Clamp a numeric above a min value, inclusive.
pub struct GetClampAbove<T, B, Min> {
    binding: Arc<B>,
    min: Arc<Min>,
    _phantom: PhantomData<fn() -> T>,
}

/// Clamp a numeric below a max value, inclusive.
pub struct GetClampBelow<T, B, Max> {
    binding: Arc<B>,
    max: Arc<Max>,
    _phantom: PhantomData<fn() -> T>,
}

/// Get an uniform random numeric value [min, max(.
///
/// This generates a new random value that is greater than or equal to `min` and less than `max`
/// every time you call `.get()` on it.
pub struct GetUniformRand<T, Min, Max> {
    min: Arc<Min>,
    max: Arc<Max>,
    _phantom: PhantomData<fn() -> T>,
}

/// Sum two numeric bindings.
pub struct GetSum<T, L, R> {
    left: Arc<L>,
    right: Arc<R>,
    _phantom: PhantomData<fn() -> T>,
}

/// Multiply two numeric bindings.
pub struct GetMul<T, L, R> {
    left: Arc<L>,
    right: Arc<R>,
    _phantom: PhantomData<fn() -> T>,
}

/// Divide two numeric bindings.
///
///*Note*: this does protected against divide by zero but just provides `Default::default()` for `T`
/// so you probably still want to protect against it.
pub struct GetDiv<T, N, D> {
    num: Arc<N>,
    den: Arc<D>,
    _phantom: PhantomData<fn() -> T>,
}

/// Negate a signed numeric binding.
pub struct GetNegate<T, B> {
    binding: Arc<B>,
    _phantom: PhantomData<fn() -> T>,
}

/// Cast one numeric binding to another.
///
/// *Note*: if the cast fails, this returns `Default::default()` of the destination value.
pub struct GetCast<I, O, B> {
    binding: Arc<B>,
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
    left: Arc<L>,
    right: Arc<R>,
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
    pub fn new(binding: Arc<B>, min: Arc<Min>, max: Arc<Max>) -> Self {
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
        num::clamp(b, min, max)
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
    pub fn new(binding: Arc<B>, min: Arc<Min>) -> Self {
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
    pub fn new(binding: Arc<B>, max: Arc<Max>) -> Self {
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

impl<T, Min, Max> GetUniformRand<T, Min, Max>
where
    T: Send,
    Min: ParamBindingGet<T>,
    Max: ParamBindingGet<T>,
{
    /// Construct a new `GetUniformRand`
    ///
    /// # Arguments
    ///
    /// * `min` - the binding for the minimum value
    /// * `max` - the binding for the maximum value
    ///
    /// # Notes
    /// The max is **exclusive** so you will never get that value in the output.
    pub fn new(min: Arc<Min>, max: Arc<Max>) -> Self {
        Self {
            min,
            max,
            _phantom: Default::default(),
        }
    }
}

impl<T, Min, Max> ParamBindingGet<T> for GetUniformRand<T, Min, Max>
where
    T: rand::distributions::uniform::SampleUniform + PartialOrd,
    Min: ParamBindingGet<T>,
    Max: ParamBindingGet<T>,
{
    fn get(&self) -> T {
        let min = self.min.get();
        let max = self.max.get();
        if min >= max {
            min
        } else {
            thread_rng().gen_range(min, max)
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
    pub fn new(left: Arc<L>, right: Arc<R>) -> Self {
        Self {
            left,
            right,
            _phantom: Default::default(),
        }
    }
}

impl<T, L, R> ParamBindingGet<T> for GetSum<T, L, R>
where
    T: std::ops::Add + num::Num,
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
    pub fn new(left: Arc<L>, right: Arc<R>) -> Self {
        Self {
            left,
            right,
            _phantom: Default::default(),
        }
    }
}

impl<T, L, R> ParamBindingGet<T> for GetMul<T, L, R>
where
    T: std::ops::Mul + num::Num,
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
    pub fn new(num: Arc<N>, den: Arc<D>) -> Self {
        Self {
            num,
            den,
            _phantom: Default::default(),
        }
    }
}

impl<T, N, D> ParamBindingGet<T> for GetDiv<T, N, D>
where
    T: std::ops::Div + num::Num + num::Zero + Default,
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
    pub fn new(binding: Arc<B>) -> Self {
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
    /// use std::sync::Arc;
    ///
    /// let f = Arc::new(23f32);
    /// let c : Arc<GetCast<f32, u8, _>> = Arc::new(GetCast::new(f.clone()));
    /// assert_eq!(23f32, f.get());
    /// assert_eq!(23u8, c.get());
    /// ```
    pub fn new(binding: Arc<B>) -> Self {
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
    pub fn new(cmp: CmpOp, left: Arc<L>, right: Arc<R>) -> Self {
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
