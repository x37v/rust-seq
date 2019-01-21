use crate::binding::BindingGetP;
use crate::context::{ChildContext, SchedContext};
use crate::graph::{ChildCount, ChildExec, GraphExec};

pub struct ClockRatio {
    mul: BindingGetP<u8>,
    div: BindingGetP<u8>,
}

impl ClockRatio {
    pub fn new(mul: BindingGetP<u8>, div: BindingGetP<u8>) -> Self {
        Self { mul, div }
    }
}

impl GraphExec for ClockRatio {
    fn exec(&mut self, context: &mut dyn SchedContext, children: &mut dyn ChildExec) -> bool {
        let div = self.div.get() as usize;

        if div > 0 && context.context_tick() % div == 0 {
            let mul = self.mul.get() as usize;
            let base_period_micros = context.base_tick_period_micros();
            let period_micros = (context.context_tick_period_micros() * div as f32) / mul as f32;
            let offset = (mul * context.context_tick()) / div;
            for i in 0..mul {
                let tick = offset + i;
                let base = ((i as f32 * period_micros) / base_period_micros) as isize;
                let mut ccontext = ChildContext::new(context, base, tick, period_micros);
                children.exec_all(&mut ccontext);
            }
        }
        //remove self if we have no children
        children.has_children()
    }

    fn children_max(&self) -> ChildCount {
        ChildCount::Inf
    }
}

mod tests {
    use super::*;
    use crate::base::{LList, SrcSink};
    use crate::context::RootContext;
    use crate::graph::ChildExec;
    use crate::time::TimeResched;
    use std::collections::VecDeque;
    use std::sync::Arc;

    pub struct CallRecord {
        base_tick: usize,
        context_tick: usize,
        context_tick_period_micros: f32,
    }

    impl CallRecord {
        pub fn new(context: &mut dyn SchedContext) -> Self {
            Self {
                base_tick: context.base_tick(),
                context_tick: context.context_tick(),
                context_tick_period_micros: context.context_tick_period_micros(),
            }
        }
    }

    pub struct Recorder {
        record: VecDeque<CallRecord>,
    }

    impl Recorder {
        pub fn new() -> Self {
            Self {
                record: VecDeque::new(),
            }
        }
    }

    impl ChildExec for Recorder {
        fn exec(&mut self, context: &mut dyn SchedContext, _index: usize) -> ChildCount {
            self.exec_all(context)
        }

        fn exec_range(
            &mut self,
            context: &mut dyn SchedContext,
            _range: std::ops::Range<usize>,
        ) -> ChildCount {
            self.exec_all(context)
        }

        fn exec_all(&mut self, context: &mut dyn SchedContext) -> ChildCount {
            self.record.push_back(CallRecord::new(context));
            self.count()
        }

        fn count(&self) -> ChildCount {
            ChildCount::Some(1)
        }

        fn has_children(&self) -> bool {
            true
        }
    }

    #[test]
    fn ratio_from_root() {
        let mut src_sink = SrcSink::new();
        let mut list = LList::new();
        let mut trig_list = LList::new();

        let mut c = RootContext::new(0usize, 44100, &mut list, &mut trig_list, &mut src_sink);

        let mut children = Recorder::new();

        let mut mul = Arc::new(1u8);
        let mut div = Arc::new(1u8);

        let mut ratio = ClockRatio::new(mul, div);
        ratio.exec(&mut c, &mut children);
        assert_eq!(1, children.record.len());

        let mut record = children.record.pop_front().unwrap();
        assert_eq!(0, record.base_tick);
        assert_eq!(0, record.context_tick);

        ratio.exec(&mut c, &mut children);
        assert_eq!(1, children.record.len());

        record = children.record.pop_front().unwrap();
        assert_eq!(0, record.base_tick);
        assert_eq!(0, record.context_tick);

        //step forward, base tick is 1
        c = RootContext::new(1usize, 44100, &mut list, &mut trig_list, &mut src_sink);
        ratio.exec(&mut c, &mut children);
        assert_eq!(1, children.record.len());

        record = children.record.pop_front().unwrap();
        assert_eq!(1, record.base_tick);
        assert_eq!(1, record.context_tick);

        //change mul, should be 2 calls per every input call
        mul = Arc::new(2u8);
        div = Arc::new(1u8);
        c = RootContext::new(0usize, 44100, &mut list, &mut trig_list, &mut src_sink);

        ratio = ClockRatio::new(mul, div);
        ratio.exec(&mut c, &mut children);
        assert_eq!(2, children.record.len());

        record = children.record.pop_front().unwrap();
        assert_eq!(0, record.base_tick);
        assert_eq!(0, record.context_tick);

        //since our parent context is a root context, we have no in-between samples at this point
        record = children.record.pop_front().unwrap();
        assert_eq!(0, record.base_tick);
        assert_eq!(1, record.context_tick);

        c = RootContext::new(1usize, 44100, &mut list, &mut trig_list, &mut src_sink);

        ratio.exec(&mut c, &mut children);
        assert_eq!(2, children.record.len());

        record = children.record.pop_front().unwrap();
        assert_eq!(1, record.base_tick);
        assert_eq!(2, record.context_tick);

        record = children.record.pop_front().unwrap();
        assert_eq!(1, record.base_tick);
        assert_eq!(3, record.context_tick);

        //change div, should be 1 call every 2 input calls
        mul = Arc::new(1u8);
        div = Arc::new(2u8);
        c = RootContext::new(0usize, 44100, &mut list, &mut trig_list, &mut src_sink);

        ratio = ClockRatio::new(mul, div);
        ratio.exec(&mut c, &mut children);
        assert_eq!(1, children.record.len());

        record = children.record.pop_front().unwrap();
        assert_eq!(0, record.base_tick);
        assert_eq!(0, record.context_tick);

        c = RootContext::new(1usize, 44100, &mut list, &mut trig_list, &mut src_sink);
        ratio.exec(&mut c, &mut children);
        assert_eq!(0, children.record.len());

        c = RootContext::new(2usize, 44100, &mut list, &mut trig_list, &mut src_sink);
        ratio.exec(&mut c, &mut children);
        assert_eq!(1, children.record.len());

        record = children.record.pop_front().unwrap();
        assert_eq!(2, record.base_tick);
        assert_eq!(1, record.context_tick);

        //mul and div, should be 2 calls every 2 inputs call
        mul = Arc::new(2u8);
        div = Arc::new(2u8);
        c = RootContext::new(0usize, 44100, &mut list, &mut trig_list, &mut src_sink);

        ratio = ClockRatio::new(mul, div);
        ratio.exec(&mut c, &mut children);
        assert_eq!(2, children.record.len());

        record = children.record.pop_front().unwrap();
        assert_eq!(0, record.base_tick);
        assert_eq!(0, record.context_tick);

        //we can compute in-between ticks
        record = children.record.pop_front().unwrap();
        assert_eq!(1, record.base_tick);
        assert_eq!(1, record.context_tick);

        c = RootContext::new(1usize, 44100, &mut list, &mut trig_list, &mut src_sink);
        ratio.exec(&mut c, &mut children);
        assert_eq!(0, children.record.len());

        c = RootContext::new(2usize, 44100, &mut list, &mut trig_list, &mut src_sink);
        ratio.exec(&mut c, &mut children);
        assert_eq!(2, children.record.len());

        record = children.record.pop_front().unwrap();
        assert_eq!(2, record.base_tick);
        assert_eq!(2, record.context_tick);

        record = children.record.pop_front().unwrap();
        assert_eq!(3, record.base_tick);
        assert_eq!(3, record.context_tick);
    }
}
