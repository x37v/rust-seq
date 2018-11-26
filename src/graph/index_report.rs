use super::*;
use base::TimeSched;
use binding::set::BindingSet;
use binding::BindingSetP;

pub struct IndexReporter {
    binding: BindingSetP<usize>,
}

impl IndexReporter {
    pub fn new_p(binding: BindingSetP<usize>) -> Arc<spinlock::Mutex<Self>> {
        Arc::new(spinlock::Mutex::new(Self { binding }))
    }
}

impl GraphIndexExec for IndexReporter {
    fn exec_index(&mut self, index: usize, context: &mut dyn SchedContext) {
        let t = TimeSched::ContextAbsolute(context.context_tick());
        context.schedule_value(t, &BindingSet::USize(index, self.binding.clone()));
    }
}
