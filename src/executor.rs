use super::*;
use binding::ParamBindingLatch;
use context::RootContext;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use trigger::Trigger;

pub struct Executor {
    list: LList<TimedFn>,
    triggers: LList<Arc<spinlock::Mutex<dyn Trigger>>>,
    trigger_list: LList<TimedTrig>,
    time_last: usize,
    ticks_per_second_last: usize,
    time: Arc<AtomicUsize>,
    schedule_receiver: Receiver<SchedFnNode>,
    src_sink: SrcSink,
}

impl Executor {
    pub fn new(
        time: Arc<AtomicUsize>,
        schedule_receiver: Receiver<SchedFnNode>,
        src_sink: SrcSink,
    ) -> Self {
        Executor {
            list: LList::new(),
            triggers: LList::new(),
            trigger_list: LList::new(),
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

    pub fn add_node(&mut self, node: SchedFnNode) {
        self.list.insert_time_sorted(node);
    }

    pub fn add_trigger(&mut self, trigger_node: TriggerNode) {
        self.triggers.push_back(trigger_node);
    }

    //signature of function is
    //time, index, block_time_start, trigger_schedule_object
    fn eval_triggers(&mut self) {
        //triggers are evaluated at the end of the run so 'now' is actually 'next'
        //so we evaluate all the triggers that happened before 'now'
        let now = self.time.load(Ordering::SeqCst);
        while let Some(trig) = self.trigger_list.pop_front_while(|n| n.time() < now) {
            //set all the values
            for vn in trig.values().iter() {
                vn.store();
            }
            if let Some(index) = trig.index() {
                let time = std::cmp::max(self.time_last, trig.time());
                //we pass a context to the trig but all it can access is the ability to trig
                let mut context = RootContext::new(
                    time,
                    self.ticks_per_second_last,
                    &mut self.list,
                    &mut self.trigger_list,
                    &mut self.src_sink,
                );
                for trig in self.triggers.iter() {
                    let trig = trig.lock();
                    if trig.trigger_index() == index {
                        trig.trigger_eval(time, &mut context);
                    }
                }
            }
            self.src_sink.dispose(trig);
        }
    }

    pub fn run(&mut self, ticks: usize, ticks_per_second: usize) {
        let now = self.time.load(Ordering::SeqCst);
        let next = now + ticks;
        self.ticks_per_second_last = ticks_per_second;

        //grab new nodes
        while let Ok(n) = self.schedule_receiver.try_recv() {
            self.add_node(n);
        }

        while let Some(mut timedfn) = self.list.pop_front_while(|n| n.time() < next) {
            let current = std::cmp::max(timedfn.time, now); //clamp to now at minimum
            let mut context = RootContext::new(
                current,
                ticks_per_second,
                &mut self.list,
                &mut self.trigger_list,
                &mut self.src_sink,
            );
            match timedfn.sched_call(&mut context) {
                TimeResched::Relative(time) | TimeResched::ContextRelative(time) => {
                    timedfn.time = current + std::cmp::max(1, time); //schedule minimum of 1 from current
                    self.add_node(timedfn);
                }
                TimeResched::None => {
                    self.src_sink.dispose(timedfn);
                }
            }
        }
        self.time_last = now;
        self.time.store(next, Ordering::SeqCst);
        self.eval_triggers();
    }
}

impl Sched for Executor {
    fn schedule(&mut self, time: TimeSched, func: SchedFn) {
        match self.src_sink.pop_node() {
            Some(mut n) => {
                n.set_time(util::add_atomic_time(&self.time, &time)); //XXX should we clamp above current time?
                n.set_func(Some(func));
                self.list.insert_time_sorted(n);
            }
            None => {
                println!("OOPS");
            }
        }
    }
}
