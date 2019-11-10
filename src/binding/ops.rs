extern crate alloc;
use crate::binding::{ParamBindingGet, ParamBindingSet};
use crate::tick::TickResched;
use core::cell::Cell;
use core::marker::PhantomData;
use core::ops::Deref;

use spin::Mutex;

/// Helper funcs
pub mod funcs {
    /// cast or return O::default() if cast fails
    pub fn cast_or_default<I, O>(i: I) -> O
    where
        I: num::NumCast,
        O: num::NumCast + Default,
    {
        if let Some(v) = O::from(i) {
            v
        } else {
            Default::default()
        }
    }

    /// if denominator equals zero, return default, otherwise return div
    pub fn div_protected<T>(num: T, den: T) -> T
    where
        T: core::ops::Div + num::Num + num::Zero + Default,
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
        T: core::ops::Rem + num::Num + num::Zero + Default,
    {
        if den.is_zero() {
            Default::default()
        } else {
            num.rem(den)
        }
    }
}

pub struct GetUnaryOp<I, O, F, B> {
    binding: B,
    func: F,
    _phantom: PhantomData<fn() -> (I, O)>,
}

pub struct GetBinaryOp<IL, IR, O, F, BL, BR> {
    left: BL,
    right: BR,
    func: F,
    _phantom: PhantomData<fn() -> (IL, IR, O)>,
}

pub struct GetIfElse<T, C, OT, OF>
where
    T: Send,
    C: ParamBindingGet<bool>,
    OT: ParamBindingGet<T>,
    OF: ParamBindingGet<T>,
{
    cmp: C,
    out_true: OT,
    out_false: OF,
    _phantom: PhantomData<fn() -> T>,
}

/// Get a value from a boxed slice of bindings, based on an index binding.
/// *Note*: if index is out of range, this returns `Default::default()` of the destination value.
pub struct GetIndexed<T, C, CT, Index> {
    bindings: C,
    index: Index,
    _phantom: PhantomData<fn() -> (T, CT)>,
}

/// Clamp a numeric binding between [min, max], inclusive.
pub struct GetClamp<T, B, Min, Max> {
    binding: B,
    min: Min,
    max: Max,
    _phantom: PhantomData<fn() -> T>,
}

/// Convert a usize into a TickResched
pub enum GetTickResched<B>
where
    B: ParamBindingGet<usize>,
{
    Relative(B),
    ContextRelative(B),
}

struct OneShotInner<T> {
    pub value: T,
    pub state: Option<()>,
}

/// Returns the value set to it only once after it is set, then returns a default value.
/// Resets state every time it is set.
pub struct OneShot<T, D> {
    default: D,
    inner: Mutex<Cell<OneShotInner<T>>>,
}

impl<T, C, OT, OF> GetIfElse<T, C, OT, OF>
where
    T: Send,
    C: ParamBindingGet<bool>,
    OT: ParamBindingGet<T>,
    OF: ParamBindingGet<T>,
{
    /// Construct a new `GetIfElse`
    ///
    /// # Arguments
    ///
    /// * `cond` - the condition to get
    /// * `out_true` - the binding to output on true
    /// * `out_false` - the binding to output on false
    pub fn new(cmp: C, out_true: OT, out_false: OF) -> Self {
        Self {
            cmp,
            out_true,
            out_false,
            _phantom: Default::default(),
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

impl<T> OneShotInner<T>
where
    T: Send + Sync + Copy,
{
    pub fn new(initial: T) -> Self {
        Self {
            value: initial,
            state: Some(()),
        }
    }

    pub fn value_once(&mut self) -> Option<T> {
        if self.state.is_some() {
            self.state = None;
            Some(self.value)
        } else {
            None
        }
    }

    pub fn value_set(&mut self, value: T) {
        self.state = Some(());
        self.value = value;
    }
}

impl<T, D> OneShot<T, D>
where
    T: Send + Sync + Copy,
    D: ParamBindingGet<T>,
{
    pub fn new(initial: T, default: D) -> Self {
        Self {
            default,
            inner: Mutex::new(Cell::new(OneShotInner::new(initial))),
        }
    }
}

impl<T, D> ParamBindingGet<T> for OneShot<T, D>
where
    T: Send + Sync + Copy,
    D: ParamBindingGet<T>,
{
    fn get(&self) -> T {
        let mut l = self.inner.lock();
        if let Some(v) = l.get_mut().value_once() {
            v
        } else {
            self.default.get()
        }
    }
}

impl<T, D> ParamBindingSet<T> for OneShot<T, D>
where
    T: Send + Sync + Copy,
    D: ParamBindingGet<T>,
{
    fn set(&self, value: T) {
        self.inner.lock().get_mut().value_set(value);
    }
}

impl<T, C, OT, OF> ParamBindingGet<T> for GetIfElse<T, C, OT, OF>
where
    T: Send,
    C: ParamBindingGet<bool>,
    OT: ParamBindingGet<T>,
    OF: ParamBindingGet<T>,
{
    fn get(&self) -> T {
        if self.cmp.get() {
            self.out_true.get()
        } else {
            self.out_false.get()
        }
    }
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

impl<B> ParamBindingGet<TickResched> for GetTickResched<B>
where
    B: ParamBindingGet<usize>,
{
    fn get(&self) -> TickResched {
        match self {
            Self::Relative(b) => TickResched::Relative(b.get()),
            Self::ContextRelative(b) => TickResched::ContextRelative(b.get()),
        }
    }
}

impl<I, O, F, B> GetUnaryOp<I, O, F, B>
where
    I: Send,
    O: Send,
    F: Fn(I) -> O + Send + Sync,
    B: ParamBindingGet<I>,
{
    pub fn new(func: F, binding: B) -> Self {
        Self {
            binding,
            func,
            _phantom: PhantomData,
        }
    }
}

impl<I, O, F, B> ParamBindingGet<O> for GetUnaryOp<I, O, F, B>
where
    I: Send,
    O: Send,
    F: Fn(I) -> O + Send + Sync,
    B: ParamBindingGet<I>,
{
    fn get(&self) -> O {
        (self.func)(self.binding.get())
    }
}

impl<IL, IR, O, F, BL, BR> GetBinaryOp<IL, IR, O, F, BL, BR>
where
    IL: Send,
    IR: Send,
    O: Send,
    F: Fn(IL, IR) -> O + Send + Sync,
    BL: ParamBindingGet<IL>,
    BR: ParamBindingGet<IR>,
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

impl<IL, IR, O, F, BL, BR> ParamBindingGet<O> for GetBinaryOp<IL, IR, O, F, BL, BR>
where
    IL: Send,
    IR: Send,
    O: Send,
    F: Fn(IL, IR) -> O + Send + Sync,
    BL: ParamBindingGet<IL>,
    BR: ParamBindingGet<IR>,
{
    fn get(&self) -> O {
        (self.func)(self.left.get(), self.right.get())
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;
    use crate::binding::atomic::*;
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
        let index = Arc::new(Mutex::new(GetBinaryOp::new(
            core::cmp::max,
            &0usize,
            &1usize,
        )));
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

    #[test]
    fn rem() {
        let l = Arc::new(AtomicUsize::new(0));
        let r = Arc::new(AtomicUsize::new(2));
        let b = GetBinaryOp::new(
            funcs::rem_protected,
            l.clone() as Arc<dyn ParamBindingGet<usize>>,
            r.clone() as Arc<dyn ParamBindingGet<usize>>,
        );
        assert_eq!(0, b.get());

        let l = l as Arc<dyn ParamBindingSet<usize>>;
        let r = r as Arc<dyn ParamBindingSet<usize>>;

        l.set(1);
        assert_eq!(1, b.get());
        l.set(2);
        assert_eq!(0, b.get());

        l.set(100);
        assert_eq!(0, b.get());
        l.set(101);
        assert_eq!(1, b.get());
        l.set(102);
        assert_eq!(0, b.get());
        l.set(103);
        assert_eq!(1, b.get());

        l.set(1);
        r.set(10);
        assert_eq!(1, b.get());
        l.set(10);
        assert_eq!(0, b.get());
    }
}
