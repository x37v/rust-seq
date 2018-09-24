use base::{LList, LNode, SchedFn, SrcSink, TimeSched, TimedFn, TimedTrig};
use binding::ValueSetP;
use util::add_clamped;

pub trait SchedContext {
    fn base_tick(&self) -> usize;
    fn context_tick(&self) -> usize;
    fn base_tick_period_micros(&self) -> f32;
    fn context_tick_period_micros(&self) -> f32;
    fn schedule(&mut self, t: TimeSched, func: SchedFn);
    fn schedule_trigger(&mut self, time: TimeSched, index: usize);
    fn schedule_value(&mut self, time: TimeSched, value: ValueSetP);
}

pub struct RootContext<'a> {
    base_tick: usize,
    base_tick_period_micros: f32,
    list: &'a mut LList<TimedFn>,
    trig_list: &'a mut LList<TimedTrig>,
    src_sink: &'a mut SrcSink,
}

pub struct ChildContext<'a> {
    parent: &'a mut dyn SchedContext,
    context_tick: usize,
    context_tick_period_micros: f32,
}

impl<'a> RootContext<'a> {
    pub fn new(
        tick: usize,
        ticks_per_second: usize,
        list: &'a mut LList<TimedFn>,
        trig_list: &'a mut LList<TimedTrig>,
        src_sink: &'a mut SrcSink,
    ) -> Self {
        let tpm = 1e6f32 / (ticks_per_second as f32);
        Self {
            base_tick: tick,
            base_tick_period_micros: tpm,
            list,
            trig_list,
            src_sink,
        }
    }

    fn to_tick(&self, time: &TimeSched) -> usize {
        match *time {
            TimeSched::Absolute(t) | TimeSched::ContextAbsolute(t) => t,
            TimeSched::Relative(t) | TimeSched::ContextRelative(t) => {
                add_clamped(self.base_tick, t)
            }
        }
    }
}

impl<'a> SchedContext for RootContext<'a> {
    fn base_tick(&self) -> usize {
        self.base_tick
    }
    fn context_tick(&self) -> usize {
        self.base_tick
    }
    fn base_tick_period_micros(&self) -> f32 {
        self.base_tick_period_micros
    }
    fn context_tick_period_micros(&self) -> f32 {
        self.base_tick_period_micros
    }
    fn schedule_trigger(&mut self, _time: TimeSched, index: usize) {
        //XXX, don't allocate
        let mut n = LNode::new_boxed(TimedTrig::default());
        n.set_time(self.base_tick); //XXX
        n.set_index(index);
        self.trig_list.insert(n, |n, o| n.time() <= o.time());
    }
    fn schedule_value(&mut self, _time: TimeSched, _value: ValueSetP) {}
    fn schedule(&mut self, time: TimeSched, func: SchedFn) {
        match self.src_sink.pop_node() {
            Some(mut n) => {
                n.set_func(Some(func));
                n.set_time(self.to_tick(&time));
                self.list.insert(n, |n, o| n.time() <= o.time());
            }
            None => {
                println!("OOPS");
            }
        }
    }
}

impl<'a> ChildContext<'a> {
    pub fn new(
        parent: &'a mut dyn SchedContext,
        context_tick: usize,
        context_tick_period_micros: f32,
    ) -> Self {
        Self {
            context_tick,
            context_tick_period_micros,
            parent,
        }
    }
}

impl<'a> SchedContext for ChildContext<'a> {
    fn base_tick(&self) -> usize {
        self.parent.base_tick()
    }
    fn context_tick(&self) -> usize {
        self.context_tick
    }
    fn base_tick_period_micros(&self) -> f32 {
        self.parent.base_tick_period_micros()
    }
    fn context_tick_period_micros(&self) -> f32 {
        self.context_tick_period_micros
    }
    fn schedule_trigger(&mut self, _time: TimeSched, _index: usize) {}
    fn schedule_value(&mut self, _time: TimeSched, _value: ValueSetP) {}
    fn schedule(&mut self, time: TimeSched, func: SchedFn) {
        //XXX translate time
        self.parent.schedule(time, func);
    }
}
