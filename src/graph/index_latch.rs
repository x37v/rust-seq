use super::*;
use binding::BindingLatchP;

pub struct IndexLatch<'a> {
    latches: Vec<BindingLatchP<'a>>,
}

impl<'a> IndexLatch<'a> {
    pub fn new(latches: Vec<BindingLatchP<'a>>) -> Self {
        Self { latches }
    }
}

impl<'a> GraphIndexExec for IndexLatch<'a> {
    fn exec_index(&mut self, index: usize, _context: &mut dyn SchedContext) {
        if index < self.latches.len() {
            self.latches[index].store();
        }
    }
}
