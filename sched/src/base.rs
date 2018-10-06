extern crate spinlock;
extern crate xnor_llist;

pub use xnor_llist::List as LList;
pub use xnor_llist::Node as LNode;

use binding::{ValueSet, ValueSetBinding, ValueSetP};
use context::{RootContext, SchedContext};
use std;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{sync_channel, Receiver, SyncSender, TryRecvError, TrySendError};
use std::sync::Arc;
use std::thread;
use util::add_clamped;

pub enum TimeSched {
    Absolute(usize),
    Relative(isize),
    ContextAbsolute(usize),
    ContextRelative(isize),
}

pub enum TimeResched {
    Relative(usize),
    ContextRelative(usize),
    None,
}

pub enum Value {
    Byte(u8),
    Int32(i32),
    Int64(i64),
    UInt32(u32),
    UInt64(u64),
    Float32(f32),
    Float64(f64),
    Char(char),
    Bool(bool),
}

pub trait Sched {
    fn schedule(&mut self, t: TimeSched, func: SchedFn);
}
pub trait SchedCall: Send {
    fn sched_call(&mut self, context: &mut dyn SchedContext) -> TimeResched;
}

pub trait TimedNodeData {
    fn set_time(&mut self, time: usize);
    fn time(&self) -> usize;
}

pub trait InsertTimeSorted<T> {
    fn insert_time_sorted(&mut self, node: Box<LNode<T>>);
}

pub trait ScheduleTrigger {
    fn schedule_trigger(&mut self, time: TimeSched, index: usize);
    fn schedule_valued_trigger(&mut self, time: TimeSched, index: usize, values: &[ValueSet]);
    fn schedule_value(&mut self, time: TimeSched, value: ValueSetP);
}

//an object to be put into a schedule and called later
pub type SchedFn = Box<dyn SchedCall>;
pub type TimedTrigNode = Box<LNode<TimedTrig>>;
pub type SchedFnNode = Box<LNode<TimedFn>>;
pub type ValueNode = Box<LNode<Option<Value>>>;

//implement sched_call for any Fn that with the correct sig
impl<F: Fn(&mut dyn SchedContext) -> TimeResched> SchedCall for F
where
    F: Send,
{
    fn sched_call(&mut self, context: &mut dyn SchedContext) -> TimeResched {
        (*self)(context)
    }
}

pub struct TimedFn {
    time: usize,
    func: Option<SchedFn>,
}

impl<T> InsertTimeSorted<T> for LList<T>
where
    T: TimedNodeData,
{
    fn insert_time_sorted(&mut self, node: Box<LNode<T>>) {
        self.insert(node, |n, o| n.time() <= o.time());
    }
}

impl TimedFn {
    pub fn set_func(&mut self, func: Option<SchedFn>) {
        self.func = func
    }
    pub fn func(&mut self) -> Option<SchedFn> {
        self.func.take()
    }
}

impl TimedNodeData for TimedFn {
    fn set_time(&mut self, time: usize) {
        self.time = time;
    }
    fn time(&self) -> usize {
        self.time
    }
}

impl Default for TimedFn {
    fn default() -> Self {
        Self {
            time: 0,
            func: None,
        }
    }
}

pub struct TimedTrig {
    time: usize,
    index: usize,
}

impl TimedTrig {
    pub fn set_index(&mut self, index: usize) {
        self.index = index;
    }
    pub fn index(&self) -> usize {
        self.index
    }
}

impl TimedNodeData for TimedTrig {
    fn set_time(&mut self, time: usize) {
        self.time = time;
    }
    fn time(&self) -> usize {
        self.time
    }
}

impl Default for TimedTrig {
    fn default() -> Self {
        Self { time: 0, index: 0 }
    }
}

pub struct TimedValueSetBinding {
    time: usize,
    binding: Box<dyn ValueSetBinding>,
}

pub struct SrcSink {
    node_cache: Receiver<SchedFnNode>,
    trig_cache: Receiver<TimedTrigNode>,
    dispose_schedule_sender: SyncSender<Box<dyn Send>>,
    updater: Option<SrcSinkUpdater>,
}

pub struct SrcSinkUpdater {
    node_cache_updater: SyncSender<SchedFnNode>,
    trig_cache_updater: SyncSender<TimedTrigNode>,
    dispose_schedule_receiver: Receiver<Box<dyn Send>>,
}

pub struct Executor {
    list: LList<TimedFn>,
    trigger_list: LList<TimedTrig>,
    value_set_list: LList<Box<TimedValueSetBinding>>,
    time_last: usize,
    time: Arc<AtomicUsize>,
    schedule_receiver: Receiver<SchedFnNode>,
    src_sink: SrcSink,
}

pub struct Scheduler {
    time: Arc<AtomicUsize>,
    executor: Option<Executor>,
    schedule_sender: SyncSender<SchedFnNode>,
    updater: Option<SrcSinkUpdater>,
    helper_handle: Option<thread::JoinHandle<()>>,
}

impl SrcSinkUpdater {
    pub fn new(
        node_cache_updater: SyncSender<SchedFnNode>,
        trig_cache_updater: SyncSender<TimedTrigNode>,
        dispose_schedule_receiver: Receiver<Box<dyn Send>>,
    ) -> Self {
        Self {
            node_cache_updater,
            trig_cache_updater,
            dispose_schedule_receiver,
        }
    }

    pub fn update(&self) -> bool {
        loop {
            match self.dispose_schedule_receiver.try_recv() {
                Ok(_) => continue,
                Err(TryRecvError::Empty) => (),
                Err(TryRecvError::Disconnected) => return false,
            }
            match self
                .node_cache_updater
                .try_send(LNode::new_boxed(Default::default()))
            {
                Ok(_) => continue,
                Err(TrySendError::Full(_)) => (),
                Err(TrySendError::Disconnected(_)) => return false,
            }
            match self
                .trig_cache_updater
                .try_send(LNode::new_boxed(Default::default()))
            {
                Ok(_) => continue,
                Err(TrySendError::Full(_)) => (),
                Err(TrySendError::Disconnected(_)) => return false,
            }
            break;
        }
        true
    }
}

impl SrcSink {
    pub fn new() -> Self {
        let (dispose_schedule_sender, dispose_schedule_receiver) = sync_channel(1024);
        let (node_cache_updater, node_cache) = sync_channel(1024);
        let (trig_cache_updater, trig_cache) = sync_channel(1024);
        Self {
            node_cache,
            trig_cache,
            dispose_schedule_sender,
            updater: Some(SrcSinkUpdater::new(
                node_cache_updater,
                trig_cache_updater,
                dispose_schedule_receiver,
            )),
        }
    }

    pub fn updater(&mut self) -> Option<SrcSinkUpdater> {
        self.updater.take()
    }

    pub fn pop_node(&mut self) -> Option<SchedFnNode> {
        self.node_cache.try_recv().ok()
    }

    pub fn pop_trig(&mut self) -> Option<TimedTrigNode> {
        self.trig_cache.try_recv().ok()
    }

    pub fn dispose(&mut self, item: Box<Send>) {
        let _ = self.dispose_schedule_sender.send(item);
    }
}

impl Default for SrcSink {
    fn default() -> SrcSink {
        SrcSink::new()
    }
}

impl Scheduler {
    pub fn new() -> Self {
        let (schedule_sender, schedule_receiver) = sync_channel(1024);
        let time = Arc::new(AtomicUsize::new(0));
        let mut src_sink = SrcSink::new();
        let updater = src_sink.updater();
        Scheduler {
            time: time.clone(),
            executor: Some(Executor {
                list: LList::new(),
                trigger_list: LList::new(),
                value_set_list: LList::new(),
                time,
                time_last: 0,
                schedule_receiver,
                src_sink,
            }),
            schedule_sender,
            updater,
            helper_handle: None,
        }
    }

    pub fn spawn_helper_threads(&mut self) {
        if let Some(updater) = self.updater.take() {
            //fill the caches, then spawn a thread to keep it updated
            updater.update();
            self.helper_handle = Some(thread::spawn(move || {
                let sleep_time = std::time::Duration::from_millis(5);
                while updater.update() {
                    thread::sleep(sleep_time);
                }
            }));
        }
    }

    pub fn executor(&mut self) -> Option<Executor> {
        self.executor.take()
    }
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl Executor {
    pub fn add_node(&mut self, node: SchedFnNode) {
        self.list.insert_time_sorted(node);
    }

    pub fn eval_triggers<F: FnMut(usize, usize)>(&mut self, func: &mut F) {
        //triggers are evaluated at the end of the run so 'now' is actually 'next'
        //so we evaluate all the triggers that happened before 'now'
        let now = self.time.load(Ordering::SeqCst);
        while let Some(trig) = self.trigger_list.pop_front_while(|n| n.time() < now) {
            func(
                std::cmp::max(self.time_last, trig.time()) - self.time_last,
                trig.index(),
            );
            self.src_sink.dispose(trig);
        }
    }

    pub fn run(&mut self, ticks: usize, ticks_per_second: usize) {
        let now = self.time.load(Ordering::SeqCst);
        let next = now + ticks;

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
    }
}

impl SchedCall for TimedFn {
    fn sched_call(&mut self, context: &mut dyn SchedContext) -> TimeResched {
        if let Some(ref mut f) = self.func {
            f.sched_call(context)
        } else {
            TimeResched::None
        }
    }
}

fn add_atomic_time(current: &Arc<AtomicUsize>, time: &TimeSched) -> usize {
    add_time(current.load(Ordering::SeqCst), time)
}

fn add_time(current: usize, time: &TimeSched) -> usize {
    match *time {
        TimeSched::Absolute(t) | TimeSched::ContextAbsolute(t) => t,
        TimeSched::Relative(t) | TimeSched::ContextRelative(t) => add_clamped(current, t),
    }
}

impl Sched for Scheduler {
    fn schedule(&mut self, time: TimeSched, func: SchedFn) {
        let f = LNode::new_boxed(TimedFn {
            func: Some(func),
            time: add_atomic_time(&self.time, &time),
        });
        self.schedule_sender.send(f).unwrap();
    }
}

impl Sched for Executor {
    fn schedule(&mut self, time: TimeSched, func: SchedFn) {
        match self.src_sink.pop_node() {
            Some(mut n) => {
                n.set_time(add_atomic_time(&self.time, &time)); //XXX should we clamp above current time?
                n.set_func(Some(func));
                self.list.insert_time_sorted(n);
            }
            None => {
                println!("OOPS");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use binding::{
        ParamBinding, ParamBindingGet, ParamBindingSet, SpinlockParamBinding,
        SpinlockValueSetBinding,
    };
    use std::thread;

    #[test]
    fn can_vec() {
        let _x: Vec<TimedFn> = (0..20).map({ |_| TimedFn::default() }).collect();
    }

    #[test]
    fn basic_test() {
        let mut s = Scheduler::new();
        s.spawn_helper_threads();

        let e = s.executor();
        assert!(e.is_some());
        s.schedule(
            TimeSched::Absolute(0),
            Box::new(move |context: &mut dyn SchedContext| {
                println!(
                    "Closure in schedule {}, {}",
                    context.base_tick(),
                    context.base_tick_period_micros()
                );
                let at = context.base_tick();
                context.schedule(
                    TimeSched::Relative(12),
                    //XXX shouldn't actually allocate this
                    Box::new(move |context: &mut dyn SchedContext| {
                        println!("inner dog {}, scheduled at {}", context.base_tick(), at);
                        context.schedule_trigger(TimeSched::Relative(0), 1);
                        TimeResched::None
                    }),
                );
                TimeResched::Relative(3)
            }),
        );

        let child = thread::spawn(move || {
            let mut e = e.unwrap();
            e.run(32, 44100);
            e.run(32, 44100);
        });

        assert!(child.join().is_ok());
    }
}
