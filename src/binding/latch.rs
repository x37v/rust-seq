use super::*;

pub struct BindingLatch<T> {
    get: BindingGetP<T>,
    set: BindingSetP<T>,
}

pub struct AggregateBindingLatch<'a> {
    latches: Vec<BindingLatchP<'a>>,
}

impl<T> BindingLatch<T> {
    pub fn new(get: BindingGetP<T>, set: BindingSetP<T>) -> Self {
        Self { get, set }
    }
}

impl<'a> AggregateBindingLatch<'a> {
    pub fn new(latches: Vec<BindingLatchP<'a>>) -> Self {
        Self { latches }
    }
}

impl<T> ParamBindingLatch for BindingLatch<T> {
    fn store(&self) {
        self.set.set(self.get.get());
    }
}

impl<'a> ParamBindingLatch for AggregateBindingLatch<'a> {
    fn store(&self) {
        for l in self.latches.iter() {
            l.store();
        }
    }
}
