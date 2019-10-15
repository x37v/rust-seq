extern crate alloc;
use crate::binding::ParamBindingGet;
use core::marker::PhantomData;
use core::ops::Deref;

use alloc::sync::Arc;
use spin::Mutex;

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
    _phantom: PhantomData<fn() -> (I, O)>,
}

/// Get a value from a boxed slice of bindings, based on an index binding.
/// *Note*: if index is out of range, this returns `Default::default()` of the destination value.
pub struct GetIndexed<T, C, CT, Index> {
    bindings: C,
    index: Index,
    _phantom: PhantomData<fn() -> (T, CT)>,
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
    /// extern crate alloc;
    /// use sched::binding::ParamBindingGet;
    /// use sched::binding::ops::GetCast;
    /// use alloc::sync::Arc;
    /// use spin::Mutex;
    ///
    /// let f: f32 = 23f32.into();
    /// let c : Arc<Mutex<dyn ParamBindingGet<u8>>> = Arc::new(Mutex::new(GetCast::new(f.clone())));
    /// assert_eq!(23f32, f.get());
    /// assert_eq!(23u8, c.get());
    /// ```
    pub fn new(binding: B) -> Self {
        Self {
            binding,
            _phantom: PhantomData,
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

impl<T, C, CT, Index> GetIndexed<T, C, CT, Index>
where
    T: Send,
    C: Deref<Target = [CT]> + Send + Sync,
    CT: Deref<Target = dyn ParamBindingGet<T>> + Send,
    Index: ParamBindingGet<usize>,
{
    pub fn new(bindings: C, index: Index) -> Self {
        Self {
            bindings,
            index,
            _phantom: PhantomData,
        }
    }
}

impl<T, C, CT, Index> ParamBindingGet<T> for GetIndexed<T, C, CT, Index>
where
    T: Send + Default,
    C: Deref<Target = [CT]> + Send + Sync,
    CT: Deref<Target = dyn ParamBindingGet<T>> + Send,
    Index: ParamBindingGet<usize>,
{
    fn get(&self) -> T {
        let i = self.index.get();
        if self.bindings.len() > i {
            self.bindings.get(i).unwrap().get()
        } else {
            Default::default()
        }
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;
    use crate::binding::*;
    use core::ops::Index;
    use core::sync::atomic::{AtomicUsize, Ordering};
    use spin::Mutex;
    use std::sync::Arc;

    static C1: &usize = &20;
    static C2: &usize = &22;

    static COLLECTION1: [&dyn ParamBindingGet<usize>; 2] = [C1, C2];

    static C3: Mutex<AtomicUsize> = Mutex::new(AtomicUsize::new(20));
    static C4: Mutex<AtomicUsize> = Mutex::new(AtomicUsize::new(22));

    static COLLECTION2: [&Mutex<dyn ParamBindingGet<usize>>; 2] = [&C3, &C4];

    #[test]
    fn indexed() {
        let index = Arc::new(AtomicUsize::new(0));
        let collection: Box<[Arc<dyn ParamBindingGet<usize>>]> = Box::new([
            Arc::new(AtomicUsize::new(10)),
            Arc::new(AtomicUsize::new(11)),
        ]);
        let indexed = GetIndexed::new(collection, index.clone() as Arc<dyn ParamBindingGet<usize>>);

        assert_eq!(10, indexed.get());
        index.store(1, Ordering::SeqCst);
        assert_eq!(11, indexed.get());

        //out of range, becomes default
        index.store(2, Ordering::SeqCst);
        assert_eq!(0, indexed.get());

        let index = Arc::new(AtomicUsize::new(0));
        let collection: Arc<[Arc<dyn ParamBindingGet<usize>>]> = Arc::new([
            Arc::new(AtomicUsize::new(10)),
            Arc::new(AtomicUsize::new(11)),
        ]);

        //can index with an op
        let index = Arc::new(Mutex::new(GetClamp::new(&0usize, &1usize, &2usize)));
        let indexed = GetIndexed::new(
            collection.clone(),
            index.clone() as Arc<Mutex<dyn ParamBindingGet<usize>>>,
        );

        let index = Arc::new(AtomicUsize::new(0));
        let indexed = GetIndexed::new(collection, index.clone() as Arc<dyn ParamBindingGet<usize>>);

        assert_eq!(10, indexed.get());
        index.store(1, Ordering::SeqCst);
        assert_eq!(11, indexed.get());

        //out of range, becomes default
        index.store(2, Ordering::SeqCst);
        assert_eq!(0, indexed.get());

        let collection = &COLLECTION1 as &[&dyn ParamBindingGet<usize>];
        let indexed = GetIndexed::new(collection, index.clone() as Arc<dyn ParamBindingGet<usize>>);

        index.store(0, Ordering::SeqCst);
        assert_eq!(20, indexed.get());
        index.store(1, Ordering::SeqCst);
        assert_eq!(22, indexed.get());

        //out of range, becomes default
        index.store(2, Ordering::SeqCst);
        assert_eq!(0, indexed.get());
    }

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
