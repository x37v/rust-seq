use binding::ParamBindingGet;
use rand::prelude::*;
use std::marker::PhantomData;
use std::sync::Arc;

///Clamp a numeric binding between a minimum and maximum value, inclusive.
pub struct ParamBindingGetClamp<T, B, Min, Max> {
    binding: Arc<B>,
    min: Arc<Min>,
    max: Arc<Max>,
    _phantom: spinlock::Mutex<PhantomData<T>>,
}

///Get an random numeric value [min, max(.
pub struct ParamBindingGetRand<T, Min, Max> {
    min: Arc<Min>,
    max: Arc<Max>,
    _phantom: spinlock::Mutex<PhantomData<T>>,
}

///Sum two numeric bindings.
pub struct ParamBindingGetSum<T, L, R> {
    left: Arc<L>,
    right: Arc<R>,
    _phantom: spinlock::Mutex<PhantomData<T>>,
}

///Multiply two numeric bindings.
pub struct ParamBindingGetMul<T, L, R> {
    left: Arc<L>,
    right: Arc<R>,
    _phantom: spinlock::Mutex<PhantomData<T>>,
}

///Negate a signed numeric binding.
pub struct ParamBindingGetNegate<T, B> {
    binding: Arc<B>,
    _phantom: spinlock::Mutex<PhantomData<T>>,
}

///Cast one numeric binding to another.
pub struct ParamBindingGetCast<B, I, O> {
    binding: Arc<B>,
    _iphantom: spinlock::Mutex<PhantomData<I>>,
    _ophantom: spinlock::Mutex<PhantomData<O>>,
}

impl<T, B, Min, Max> ParamBindingGetClamp<T, B, Min, Max>
where
    T: Send + Copy + PartialOrd,
    B: ParamBindingGet<T>,
    Min: ParamBindingGet<T>,
    Max: ParamBindingGet<T>,
{
    pub fn new(binding: Arc<B>, min: Arc<Min>, max: Arc<Max>) -> Self {
        Self {
            binding,
            min,
            max,
            _phantom: Default::default(),
        }
    }
}

impl<T, B, Min, Max> ParamBindingGet<T> for ParamBindingGetClamp<T, B, Min, Max>
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

impl<T, Min, Max> ParamBindingGetRand<T, Min, Max>
where
    T: Send,
    Min: ParamBindingGet<T>,
    Max: ParamBindingGet<T>,
{
    pub fn new(min: Arc<Min>, max: Arc<Max>) -> Self {
        Self {
            min,
            max,
            _phantom: Default::default(),
        }
    }
}

impl<T, Min, Max> ParamBindingGet<T> for ParamBindingGetRand<T, Min, Max>
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

impl<T, L, R> ParamBindingGetSum<T, L, R>
where
    T: Send,
    L: ParamBindingGet<T>,
    R: ParamBindingGet<T>,
{
    pub fn new(left: Arc<L>, right: Arc<R>) -> Self {
        Self {
            left,
            right,
            _phantom: Default::default(),
        }
    }
}

impl<T, L, R> ParamBindingGet<T> for ParamBindingGetSum<T, L, R>
where
    T: std::ops::Add + num::Num,
    L: ParamBindingGet<T>,
    R: ParamBindingGet<T>,
{
    fn get(&self) -> T {
        self.left.get().add(self.right.get())
    }
}

impl<T, L, R> ParamBindingGetMul<T, L, R>
where
    T: Send,
    L: ParamBindingGet<T>,
    R: ParamBindingGet<T>,
{
    pub fn new(left: Arc<L>, right: Arc<R>) -> Self {
        Self {
            left,
            right,
            _phantom: Default::default(),
        }
    }
}

impl<T, L, R> ParamBindingGet<T> for ParamBindingGetMul<T, L, R>
where
    T: std::ops::Mul + num::Num,
    L: ParamBindingGet<T>,
    R: ParamBindingGet<T>,
{
    fn get(&self) -> T {
        self.left.get().mul(self.right.get())
    }
}

impl<T, B> ParamBindingGetNegate<T, B>
where
    T: Send,
    B: ParamBindingGet<T>,
{
    pub fn new(binding: Arc<B>) -> Self {
        Self {
            binding,
            _phantom: Default::default(),
        }
    }
}

impl<T, B> ParamBindingGet<T> for ParamBindingGetNegate<T, B>
where
    T: num::Signed,
    B: ParamBindingGet<T>,
{
    fn get(&self) -> T {
        -self.binding.get()
    }
}

impl<B, I, O> ParamBindingGetCast<B, I, O>
where
    I: Send,
    O: Send,
    B: ParamBindingGet<I>,
{
    pub fn new(binding: Arc<B>) -> Self {
        Self {
            binding,
            _iphantom: Default::default(),
            _ophantom: Default::default(),
        }
    }
}

impl<B, I, O> ParamBindingGet<O> for ParamBindingGetCast<B, I, O>
where
    I: num::NumCast,
    O: num::NumCast,
    B: ParamBindingGet<I>,
{
    fn get(&self) -> O {
        O::from(self.binding.get()).unwrap()
    }
}
