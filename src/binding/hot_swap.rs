use super::*;
use alloc::sync::Arc;

pub struct HotSwapGet<T> {
    value: Arc<dyn ParamBindingGet<T>>,
}

pub struct HotSwapSet<T> {
    value: Arc<dyn ParamBindingSet<T>>,
}

impl<T> HotSwapGet<T> {
    pub fn swap(&mut self, value: Arc<dyn ParamBindingGet<T>>) {
        self.value = value;
    }
}

impl<T> HotSwapSet<T> {
    pub fn swap(&mut self, value: Arc<dyn ParamBindingSet<T>>) {
        self.value = value;
    }
}

impl<T> ParamBindingGet<T> for HotSwapGet<T> {
    fn get(&self) -> T {
        self.value.get()
    }
}

impl<T> ParamBindingSet<T> for HotSwapSet<T> {
    fn set(&self, value: T) {
        self.value.set(value);
    }
}
