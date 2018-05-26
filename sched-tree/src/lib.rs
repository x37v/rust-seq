extern crate sched;

use sched::{ExecSched, SchedCall, SchedFn, TimeResched};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct Clock<SrcSnk, Context> {
    tick_period: Arc<AtomicUsize>,
    sched: SchedFn<SrcSnk, Context>,
}

impl<SrcSnk, Context> SchedCall<SrcSnk, Context> for Clock<SrcSnk, Context> {
    fn sched_call(
        &mut self,
        s: &mut ExecSched<SrcSnk, Context>,
        context: &mut Context,
    ) -> TimeResched {
        match self.sched.sched_call(s, context) {
            TimeResched::None => TimeResched::None,
            _ => TimeResched::ContextRelative(self.tick_period.load(Ordering::SeqCst)),
        }
    }
}

impl<SrcSnk, Context> Clock<SrcSnk, Context> {
    pub fn new(tick_period: Arc<AtomicUsize>, sched: SchedFn<SrcSnk, Context>) -> Self {
        Clock { tick_period, sched }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
