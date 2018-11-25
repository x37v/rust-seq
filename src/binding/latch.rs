use super::*;

pub struct BindingLatch<T> {
    get: BindingGetP<T>,
    set: BindingSetP<T>,
}

pub struct AggregateBindingLatch {
    latches: Vec<Arc<dyn ParamBindingLatch>>,
}

impl<T> BindingLatch<T> {
    pub fn new(get: BindingGetP<T>, set: BindingSetP<T>) -> Self {
        Self { get, set }
    }
}

impl AggregateBindingLatch {
    pub fn new(latches: Vec<Arc<dyn ParamBindingLatch>>) -> Self {
        Self { latches }
    }
}

impl<T> ParamBindingLatch for BindingLatch<T> {
    fn store(&self) {
        self.set.set(self.get.get());
    }
}

impl ParamBindingLatch for AggregateBindingLatch {
    fn store(&self) {
        for l in self.latches.iter() {
            l.store();
        }
    }
}
