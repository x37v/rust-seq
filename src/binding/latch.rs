use super::*;
use core::marker::PhantomData;

pub struct BindingLatch<T, Get, Set> {
    get: Get,
    set: Set,
    _phantom: PhantomData<fn() -> T>,
}

impl<T, Get, Set> BindingLatch<T, Get, Set> {
    pub fn new(get: Get, set: Set) -> Self {
        Self {
            get,
            set,
            _phantom: PhantomData,
        }
    }
}

impl<T, Get, Set> ParamBindingLatch for BindingLatch<T, Get, Set>
where
    T: Send + Copy,
    Get: ParamBindingGet<T>,
    Set: ParamBindingSet<T>,
{
    fn store(&self) {
        self.set.set(self.get.get());
    }
}

/*
 * TODO ?
pub struct AggregateBindingLatch<'a> {
    latches: Vec<BindingLatchP<'a>>,
}


impl<'a> AggregateBindingLatch<'a> {
    pub fn new(latches: Vec<BindingLatchP<'a>>) -> Self {
        Self { latches }
    }
}

impl<'a> ParamBindingLatch for AggregateBindingLatch<'a> {
    fn store(&self) {
        for l in self.latches.iter() {
            l.store();
        }
    }
}
*/
