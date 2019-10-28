use super::*;
use crate::binding::ParamBindingLatch;
use crate::context::RootContext;
use crate::pqueue::PriorityQueue;
use crate::ptr::{SShrPtr, ShrPtr, UniqPtr};
use crate::time::{TimeResched, TimeSched};
use crate::trigger::Trigger;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::Receiver;
use xnor_llist::{List, Node};

//XXX use TrigCallPtr
pub struct Executor<SPQ, TPQ>
where
    SPQ: PriorityQueue<usize, SchedFn>,
    TPQ: PriorityQueue<usize, TrigCallPtr>,
{
    schedule: SPQ,
    trigger_schedule: TPQ,
    triggers: List<TrigPtr>,
    time_last: usize,
    ticks_per_second_last: usize,
    time: ShrPtr<AtomicUsize>,
    schedule_receiver: Receiver<(usize, SchedFn)>,
    src_sink: SrcSink,
}

impl<SPQ, TPQ> Executor<SPQ, TPQ>
where
    SPQ: PriorityQueue<usize, SchedFn>,
    TPQ: PriorityQueue<usize, TrigCallPtr>,
{
    pub fn new(
        schedule: SPQ,
        trigger_schedule: TPQ,
        time: ShrPtr<AtomicUsize>,
        schedule_receiver: Receiver<(usize, SchedFn)>,
        src_sink: SrcSink,
    ) -> Self {
        Executor {
            schedule,
            trigger_schedule,
            triggers: List::new(),
            time,
            time_last: 0,
            ticks_per_second_last: 0,
            schedule_receiver,
            src_sink,
        }
    }

    pub fn time_last(&self) -> usize {
        self.time_last
    }

    pub fn schedule(&mut self, tick: usize, func: SchedFn) {
        self.schedule.insert(tick, func);
    }

    pub fn add_trigger(&mut self, _trigger: TrigPtr) {
        //XXX self.triggers.push_back(trigger);
    }

    //signature of function is
    //time, index, block_time_start, trigger_schedule_object
    fn eval_triggers(&mut self) {
        //triggers are evaluated at the end of the run so 'now' is actually 'next'
        //so we evaluate all the triggers that happened before 'now'
        let now = self.time.load(Ordering::SeqCst);
        while let Some((time, mut trig)) = self.trigger_schedule.pop_lt(now) {
            trig.latch_values();
            if let Some(index) = trig.index() {
                let time = std::cmp::max(self.time_last, time);
                //we pass a context to the trig but all it can access is the ability to trig
                let mut context = RootContext::new(
                    time,
                    self.ticks_per_second_last,
                    &mut self.schedule,
                    &mut self.trigger_schedule,
                    &mut self.src_sink,
                );
                for trig in self.triggers.iter() {
                    let trig = trig.lock();
                    if trig.trigger_index() == index {
                        trig.trigger_eval(time, &mut context);
                    }
                }
            }
            //XXX self.src_sink.dispose(trig);
        }
    }

    pub fn run(&mut self, ticks: usize, ticks_per_second: usize) {
        let now = self.time.load(Ordering::SeqCst);
        let next = now + ticks;
        self.ticks_per_second_last = ticks_per_second;

        //grab new events
        while let Ok((t, n)) = self.schedule_receiver.try_recv() {
            self.schedule.insert(t, n);
        }

        while let Some((time, mut func)) = self.schedule.pop_lt(next) {
            let current = std::cmp::max(time, now); //clamp to now at minimum
            let mut context = RootContext::new(
                current,
                ticks_per_second,
                &mut self.schedule,
                &mut self.trigger_schedule,
                &mut self.src_sink,
            );
            match func.sched_call(&mut context) {
                TimeResched::Relative(time) | TimeResched::ContextRelative(time) => {
                    let time = current + std::cmp::max(1, time); //schedule minimum of 1 from current
                    self.schedule(time, func);
                }
                TimeResched::None => {
                    //XXX TODO self.src_sink.dispose(func);
                }
            }
        }
        self.time_last = now;
        self.time.store(next, Ordering::SeqCst);
        self.eval_triggers();
    }
}

impl<SPQ, TPQ> Sched for Executor<SPQ, TPQ>
where
    SPQ: PriorityQueue<usize, SchedFn>,
    TPQ: PriorityQueue<usize, TrigCallPtr>,
{
    fn schedule(&mut self, time: TimeSched, func: SchedFn) {
        //XXX should we clamp above current time?
        self.schedule
            .insert(util::add_atomic_time(&self.time, &time), func);
    }
}
