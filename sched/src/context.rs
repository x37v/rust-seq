use base::{
    InsertTimeSorted, LList, SchedFn, ScheduleTrigger, SrcSink, TimeSched, TimedFn, TimedNodeData,
    TimedTrig,
};
use binding::{ValueSet, ValueSetP};
use util::add_clamped;

pub trait SchedContext: ScheduleTrigger {
    fn base_tick(&self) -> usize;
    fn context_tick(&self) -> usize;
    fn base_tick_period_micros(&self) -> f32;
    fn context_tick_period_micros(&self) -> f32;
    fn schedule(&mut self, t: TimeSched, func: SchedFn);
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
    fn schedule(&mut self, time: TimeSched, func: SchedFn) {
        if let Some(mut n) = self.src_sink.pop_node() {
            n.set_func(Some(func));
            n.set_time(self.to_tick(&time));
            self.list.insert_time_sorted(n);
        } else {
            println!("OOPS");
        }
    }
}

impl<'a> ScheduleTrigger for RootContext<'a> {
    fn schedule_trigger(&mut self, time: TimeSched, index: usize) {
        if let Some(mut n) = self.src_sink.pop_trig() {
            n.set_index(Some(index));
            n.set_time(self.to_tick(&time));
            self.trig_list.insert_time_sorted(n);
        } else {
            println!("OOPS");
        }
    }
    fn schedule_valued_trigger(&mut self, time: TimeSched, index: usize, values: &[ValueSet]) {
        //XXX implement
    }
    fn schedule_value(&mut self, _time: TimeSched, _value: ValueSetP) {}
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
    fn schedule(&mut self, time: TimeSched, func: SchedFn) {
        //XXX translate time
        self.parent.schedule(time, func);
    }
}

impl<'a> ScheduleTrigger for ChildContext<'a> {
    fn schedule_trigger(&mut self, time: TimeSched, index: usize) {
        self.parent.schedule_trigger(time, index); //XXX translate time
    }
    fn schedule_valued_trigger(&mut self, time: TimeSched, index: usize, values: &[ValueSet]) {
        self.parent.schedule_valued_trigger(time, index, values); //XXX translate time
    }
    fn schedule_value(&mut self, _time: TimeSched, _value: ValueSetP) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use base::{LList, Sched, Scheduler, SrcSink, TimeSched};
    use binding::{ParamBindingSet, SpinlockParamBinding};
    use context::{RootContext, SchedContext};
    use std;
    use std::thread;
    #[test]
    fn works() {
        let mut src_sink = SrcSink::new();
        let mut list = LList::new();
        let mut trig_list = LList::new();

        let mut c = RootContext::new(0, 0, &mut list, &mut trig_list, &mut src_sink);
        let fbinding = SpinlockParamBinding::new_p(0f32);
        let ibinding = SpinlockParamBinding::new_p(0);
        c.schedule_valued_trigger(
            TimeSched::Relative(0),
            0,
            &[ValueSet::F32(3.0, fbinding), ValueSet::I32(2084, ibinding)],
        );
    }
}
