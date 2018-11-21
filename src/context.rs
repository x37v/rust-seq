use base::{
    InsertTimeSorted, LList, SchedFn, ScheduleTrigger, SrcSink, TimeResched, TimeSched, TimedFn,
    TimedNodeData, TimedTrig,
};
use binding::ValueSet;
use trigger::TriggerId;
use util::add_clamped;

pub trait SchedContext: ScheduleTrigger {
    fn base_tick(&self) -> usize;
    fn context_tick(&self) -> usize;
    fn base_tick_period_micros(&self) -> f32;
    fn context_tick_period_micros(&self) -> f32;
    fn schedule(&mut self, t: TimeSched, func: SchedFn);
    fn as_schedule_trigger_mut(&mut self) -> &mut ScheduleTrigger;
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
    parent_tick_offset: isize,
    context_tick: usize,
    context_tick_period_micros: f32,
}

fn translate_tick(dest_micros_per_tick: f32, src_micros_per_tick: f32, src_tick: isize) -> isize {
    if dest_micros_per_tick <= 0f32 {
        0isize
    } else {
        (src_tick as f32 * src_micros_per_tick / dest_micros_per_tick) as isize
    }
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
    fn as_schedule_trigger_mut(&mut self) -> &mut ScheduleTrigger {
        self
    }
}

impl<'a> ScheduleTrigger for RootContext<'a> {
    fn schedule_trigger(&mut self, time: TimeSched, index: TriggerId) {
        if let Some(mut n) = self.src_sink.pop_trig() {
            n.set_index(Some(index));
            n.set_time(self.to_tick(&time));
            self.trig_list.insert_time_sorted(n);
        } else {
            println!("OOPS");
        }
    }
    fn schedule_valued_trigger(&mut self, time: TimeSched, index: TriggerId, values: &[ValueSet]) {
        if let Some(mut n) = self.src_sink.pop_trig() {
            n.set_index(Some(index));
            n.set_time(self.to_tick(&time));
            for v in values {
                if let Some(mut vn) = self.src_sink.pop_value_set() {
                    **vn = v.clone();
                    n.add_value(vn);
                } else {
                    println!("OOPS");
                    return;
                }
            }
            self.trig_list.insert_time_sorted(n);
        } else {
            println!("OOPS");
        }
    }
    fn schedule_value(&mut self, time: TimeSched, value: &ValueSet) {
        if let Some(mut n) = self.src_sink.pop_trig() {
            n.set_index(None);
            n.set_time(self.to_tick(&time));
            if let Some(mut vn) = self.src_sink.pop_value_set() {
                **vn = value.clone();
                n.add_value(vn);
                self.trig_list.insert_time_sorted(n);
            } else {
                println!("OOPS");
            }
        } else {
            println!("OOPS");
        }
    }

    //at the root, context and non context are the same
    fn add_time(&self, time: &TimeSched, dur: &TimeResched) -> TimeSched {
        let mut offset: usize = 0;
        match dur {
            TimeResched::None => (),
            TimeResched::Relative(ref t) => offset = *t,
            TimeResched::ContextRelative(ref t) => offset = *t,
        }
        match time {
            TimeSched::Absolute(ref t) => TimeSched::Absolute(t + offset),
            TimeSched::Relative(ref t) => TimeSched::Relative(t + offset as isize),
            TimeSched::ContextAbsolute(ref t) => TimeSched::ContextAbsolute(t + offset),
            TimeSched::ContextRelative(ref t) => TimeSched::ContextRelative(t + offset as isize),
        }
    }
}

impl<'a> ChildContext<'a> {
    pub fn new(
        parent: &'a mut dyn SchedContext,
        parent_tick_offset: isize,
        context_tick: usize,
        context_tick_period_micros: f32,
    ) -> Self {
        Self {
            parent,
            parent_tick_offset,
            context_tick,
            context_tick_period_micros,
        }
    }

    pub fn translate_time(&self, time: &TimeSched) -> TimeSched {
        match *time {
            TimeSched::Absolute(t) => TimeSched::Absolute(t),
            TimeSched::Relative(t) => TimeSched::Absolute(add_clamped(self.base_tick(), t)),
            TimeSched::ContextAbsolute(t) => {
                let offset = translate_tick(
                    self.base_tick_period_micros(),
                    self.context_tick_period_micros(),
                    t as isize - self.context_tick() as isize,
                );
                TimeSched::Absolute(add_clamped(self.base_tick(), offset))
            }
            TimeSched::ContextRelative(t) => {
                //convert to base ticks, absolute from our base tick
                let offset = translate_tick(
                    self.base_tick_period_micros(),
                    self.context_tick_period_micros(),
                    t,
                );
                TimeSched::Absolute(add_clamped(self.base_tick(), offset))
            }
        }
    }
}

impl<'a> SchedContext for ChildContext<'a> {
    fn base_tick(&self) -> usize {
        add_clamped(self.parent.base_tick(), self.parent_tick_offset)
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
        let time = self.translate_time(&time);
        self.parent.schedule(time, func);
    }

    fn as_schedule_trigger_mut(&mut self) -> &mut ScheduleTrigger {
        self
    }
}

impl<'a> ScheduleTrigger for ChildContext<'a> {
    fn schedule_trigger(&mut self, time: TimeSched, index: TriggerId) {
        self.parent
            .schedule_trigger(self.translate_time(&time), index);
    }
    fn schedule_valued_trigger(&mut self, time: TimeSched, index: TriggerId, values: &[ValueSet]) {
        self.parent
            .schedule_valued_trigger(self.translate_time(&time), index, values);
    }
    fn schedule_value(&mut self, time: TimeSched, value: &ValueSet) {
        self.parent
            .schedule_value(self.translate_time(&time), value);
    }

    fn add_time(&self, time: &TimeSched, dur: &TimeResched) -> TimeSched {
        self.parent.add_time(&self.translate_time(&time), dur)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use base::{LList, SrcSink, TimeSched};
    use binding::SpinlockParamBinding;
    use context::RootContext;
    #[test]
    fn works() {
        let mut src_sink = SrcSink::new();
        let mut list = LList::new();
        let mut trig_list = LList::new();

        let mut c = RootContext::new(0, 0, &mut list, &mut trig_list, &mut src_sink);
        let fbinding = SpinlockParamBinding::new_p(0f32);
        let ibinding = SpinlockParamBinding::new_p(0);
        let trig = TriggerId::new();
        c.schedule_valued_trigger(
            TimeSched::Relative(0),
            trig,
            &[ValueSet::F32(3.0, fbinding), ValueSet::I32(2084, ibinding)],
        );
    }
}
