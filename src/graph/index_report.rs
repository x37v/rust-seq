use super::*;
use crate::base::TimeSched;
use crate::binding::set::BindingSet;
use crate::binding::BindingSetP;

pub struct IndexReporter {
    binding: BindingSetP<usize>,
}

impl IndexReporter {
    pub fn new(binding: BindingSetP<usize>) -> Self {
        Self { binding }
    }
}

impl GraphIndexExec for IndexReporter {
    fn exec_index(&mut self, index: usize, context: &mut dyn SchedContext) {
        let t = TimeSched::ContextAbsolute(context.context_tick());
        context.schedule_value(t, &BindingSet::USize(index, self.binding.clone()));
    }
}
