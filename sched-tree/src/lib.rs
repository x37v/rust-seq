extern crate sched;

use sched::{ExecSched, SchedCall, SchedFn, TimeResched, ContextBase};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct Clock<SrcSnk, Context> {
    period_micros: Arc<AtomicUsize>,
    sched: SchedFn<SrcSnk, Context>,
}

impl<SrcSnk, Context: ContextBase> SchedCall<SrcSnk, Context> for Clock<SrcSnk, Context> {
    fn sched_call(
        &mut self,
        s: &mut ExecSched<SrcSnk, Context>,
        context: &mut Context,
    ) -> TimeResched {
        match self.sched.sched_call(s, context) {
            TimeResched::None => TimeResched::None,
            _ => TimeResched::ContextRelative(std::cmp::max(1, (self.period_micros.load(Ordering::SeqCst) * context.ticks_per_second()) / 1_000_000usize)),
        }
    }
}

impl<SrcSnk, Context> Clock<SrcSnk, Context> {
    pub fn new(period_micros: Arc<AtomicUsize>, sched: SchedFn<SrcSnk, Context>) -> Self {
        Clock { period_micros, sched }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
