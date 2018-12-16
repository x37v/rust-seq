use binding::BindingGetP;
use context::{ChildContext, SchedContext};
use graph::{ChildCount, ChildExec, GraphExec};

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
            let period_micros = (context.context_tick_period_micros() * div as f32) / mul as f32;
            let offset = (mul * context.context_tick()) / div;
            for i in 0..mul {
                let tick = offset + i;
                let mut ccontext = ChildContext::new(
                    context,
                    (i as f32 * period_micros) as isize,
                    tick,
                    period_micros,
                );
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
    use base::{LList, SrcSink, TimeResched};
    use context::RootContext;
    use graph::ChildExec;
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
        record: Vec<CallRecord>,
    }

    impl Recorder {
        pub fn new() -> Self {
            Self { record: Vec::new() }
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
            self.record.push(CallRecord::new(context));
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
    fn ratios() {
        let mut src_sink = SrcSink::new();
        let mut list = LList::new();
        let mut trig_list = LList::new();

        let tick = 0usize;

        let mut c = RootContext::new(
            tick as usize,
            44100,
            &mut list,
            &mut trig_list,
            &mut src_sink,
        );

        let mut children = Recorder::new();

        let mut mul = Arc::new(1u8);
        let mut div = Arc::new(1u8);

        let mut ratio = ClockRatio::new(mul, div);
        ratio.exec(&mut c, &mut children);
        assert_eq!(1, children.record.len());

        let mut record = children.record.pop().unwrap();
        assert_eq!(0, record.base_tick);
        assert_eq!(0, record.context_tick);

        ratio.exec(&mut c, &mut children);
        assert_eq!(1, children.record.len());

        record = children.record.pop().unwrap();
        assert_eq!(0, record.base_tick);
        assert_eq!(0, record.context_tick);

        //change mul
        mul = Arc::new(2u8);
        div = Arc::new(1u8);

        ratio = ClockRatio::new(mul, div);
        ratio.exec(&mut c, &mut children);
        assert_eq!(2, children.record.len());

        record = children.record.pop().unwrap();
        assert_eq!(0, record.base_tick);
        assert_eq!(0, record.context_tick);

        record = children.record.pop().unwrap();
        assert_eq!(0, record.base_tick);
        assert_eq!(1, record.context_tick);
    }
}
