use super::*;
use crate::binding::ParamBindingLatch;

pub struct IndexLatch<T>
where
    T: ParamBindingLatch,
{
    latches: Vec<T>,
}

impl<T> IndexLatch<T>
where
    T: ParamBindingLatch,
{
    pub fn new(latches: Vec<T>) -> Self {
        Self { latches }
    }
}

impl<T> GraphIndexExec for IndexLatch<T>
where
    T: ParamBindingLatch,
{
    fn exec_index(&mut self, index: usize, _context: &mut dyn SchedContext) {
        if index < self.latches.len() {
            self.latches[index].store();
        }
    }
}

pub struct IndexSliceLatch<'a> {
    latches: &'a [&'a dyn ParamBindingLatch],
}

impl<'a> IndexSliceLatch<'a> {
    pub fn new(latches: &'a [&'a dyn ParamBindingLatch]) -> Self {
        Self { latches }
    }
}

impl<'a> GraphIndexExec for IndexSliceLatch<'a> {
    fn exec_index(&mut self, index: usize, _context: &mut dyn SchedContext) {
        if index < self.latches.len() {
            self.latches[index].store();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binding::latch::BindingLatch;
    use crate::binding::{ParamBindingGet, ParamBindingLatch, ParamBindingSet};
    use core::sync::atomic::AtomicUsize;

    static TEST_ATOMIC0: AtomicUsize = AtomicUsize::new(0);
    static TEST_ATOMIC1: AtomicUsize = AtomicUsize::new(1);
    static TEST_ATOMIC2: AtomicUsize = AtomicUsize::new(2);
    static TEST_ATOMIC3: AtomicUsize = AtomicUsize::new(3);

    static LATCH0: BindingLatch<
        usize,
        &'static dyn ParamBindingGet<usize>,
        &'static dyn ParamBindingSet<usize>,
    > = BindingLatch::new(
        &TEST_ATOMIC0 as &'static dyn ParamBindingGet<usize>,
        &TEST_ATOMIC1 as &'static dyn ParamBindingSet<usize>,
    );
    static LATCH1: BindingLatch<
        usize,
        &'static dyn ParamBindingGet<usize>,
        &'static dyn ParamBindingSet<usize>,
    > = BindingLatch::new(
        &TEST_ATOMIC2 as &'static dyn ParamBindingGet<usize>,
        &TEST_ATOMIC3 as &'static dyn ParamBindingSet<usize>,
    );

    #[test]
    fn slice() {
        let _n = IndexSliceLatch::new(&[
            &LATCH0 as &'static dyn ParamBindingLatch,
            &LATCH1 as &'static dyn ParamBindingLatch,
        ]);
    }
}
