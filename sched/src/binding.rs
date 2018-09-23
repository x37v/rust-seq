extern crate spinlock;
extern crate xnor_llist;

pub use xnor_llist::List as LList;
pub use xnor_llist::Node as LNode;

use std::cell::Cell;
use std::sync::Arc;

pub type ValueSetP = Box<dyn ValueSetBinding>;
pub type BindingP<T> = Arc<dyn ParamBinding<T>>;

pub trait ParamBinding<T>: Send + Sync {
    fn set(&self, value: T);
    fn get(&self) -> T;
}

pub trait ValueSetBinding: Send {
    //store the value into the binding
    fn store(&self);
}

pub struct SpinlockParamBinding<T: Copy> {
    lock: spinlock::Mutex<Cell<T>>,
}

pub struct SpinlockValueSetBinding<T: Copy> {
    binding: BindingP<T>,
    value: T,
}

impl<T: Copy> SpinlockParamBinding<T> {
    pub fn new(value: T) -> Self {
        SpinlockParamBinding {
            lock: spinlock::Mutex::new(Cell::new(value)),
        }
    }
}

impl<T: Copy + Send> ParamBinding<T> for SpinlockParamBinding<T> {
    fn set(&self, value: T) {
        self.lock.lock().set(value);
    }

    fn get(&self) -> T {
        self.lock.lock().get()
    }
}

impl<T: Copy> SpinlockValueSetBinding<T> {
    pub fn new(binding: BindingP<T>, value: T) -> Self {
        SpinlockValueSetBinding { binding, value }
    }
}

impl<T: Copy + Send> ValueSetBinding for SpinlockValueSetBinding<T> {
    fn store(&self) {
        self.binding.set(self.value);
    }
}
