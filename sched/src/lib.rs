#![feature(nll)]

pub extern crate spinlock;
pub extern crate xnor_llist;

pub use xnor_llist::List as LList;
pub use xnor_llist::Node as LNode;

use std::cell::Cell;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{sync_channel, Receiver, SyncSender, TryRecvError, TrySendError};
use std::sync::Arc;
use std::thread;

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

pub trait ParamBinding<T> {
    fn set(&self, value: T);
    fn get(&self) -> T;
}

pub struct SpinlockParamBinding<T: Copy> {
    lock: spinlock::Mutex<Cell<T>>,
}

impl<T: Copy> SpinlockParamBinding<T> {
    pub fn new(value: T) -> Self {
        SpinlockParamBinding {
            lock: spinlock::Mutex::new(Cell::new(value)),
        }
    }
}

impl<T: Copy> ParamBinding<T> for SpinlockParamBinding<T> {
    fn set(&self, value: T) {
        self.lock.lock().set(value);
    }

    fn get(&self) -> T {
        self.lock.lock().get()
    }
}

pub type BindingP<T> = Arc<SpinlockParamBinding<T>>;

pub trait ValueSetBinding: Send {
    //store the value into the binding
    fn store(&self);
}

pub struct SpinlockValueSetBinding<T: Copy> {
    binding: BindingP<T>,
    value: T,
}

impl<T: Copy> SpinlockValueSetBinding<T> {
    pub fn new(binding: BindingP<T>, value: T) -> Self {
        SpinlockValueSetBinding { binding, value }
    }
}

impl<T: Copy + Send> ValueSetBinding for SpinlockValueSetBinding<T> {
    fn store(&self) {
        self.binding.set(self.value);
    }
}

pub trait Sched {
    fn schedule(&mut self, t: TimeSched, func: SchedFn);
}

pub trait SchedContext {
    fn base_tick(&self) -> usize;
    fn context_tick(&self) -> usize;
    fn base_tick_period_micros(&self) -> f32;
    fn context_tick_period_micros(&self) -> f32;
    fn trigger(&mut self, time: TimeSched, index: usize);
    fn schedule(&mut self, t: TimeSched, func: SchedFn);
}

//an object to be put into a schedule and called later
pub type SchedFn = Box<dyn SchedCall>;

pub trait SchedCall: Send {
    fn sched_call(&mut self, context: &mut dyn SchedContext) -> TimeResched;
}

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
pub type SchedFnNode = Box<LNode<TimedFn>>;

impl Default for TimedFn {
    fn default() -> Self {
        TimedFn {
            time: 0,
            func: None,
        }
    }
}

pub struct TimedValueSetBinding {
    time: usize,
    binding: Box<dyn ValueSetBinding>,
}

pub struct SrcSink {
    node_cache: Receiver<SchedFnNode>,
    dispose_schedule_sender: SyncSender<Box<dyn Send>>,
    updater: Option<SrcSinkUpdater>,
}

pub struct SrcSinkUpdater {
    node_cache_updater: SyncSender<SchedFnNode>,
    dispose_schedule_receiver: Receiver<Box<dyn Send>>,
}

pub struct Executor {
    list: LList<TimedFn>,
    trigger_list: LList<(usize, usize)>,
    value_set_list: LList<Box<TimedValueSetBinding>>,
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

pub struct Context<'a> {
    base_tick: usize,
    context_tick: usize,
    base_tick_period_micros: f32,
    context_tick_period_micros: f32,
    list: &'a mut LList<TimedFn>,
    context_list: Option<&'a mut LList<TimedFn>>,
    node_cache: &'a mut Receiver<SchedFnNode>,
    trigger_sender: &'a mut SyncSender<(usize, usize)>,
}

impl<'a> Context<'a> {
    fn new_root(
        tick: usize,
        ticks_per_second: usize,
        list: &'a mut LList<TimedFn>,
        node_cache: &'a mut Receiver<SchedFnNode>,
        trigger_sender: &'a mut SyncSender<(usize, usize)>,
    ) -> Self {
        let tpm = 1e6f32 / (ticks_per_second as f32);
        Context {
            base_tick: tick,
            context_tick: tick,
            base_tick_period_micros: tpm,
            context_tick_period_micros: tpm,
            list,
            context_list: None,
            node_cache,
            trigger_sender,
        }
    }

    pub fn pop_node(&mut self) -> Option<SchedFnNode> {
        self.node_cache.try_recv().ok()
    }

    fn list_and_tick(&mut self, time: &TimeSched) -> (&mut LList<TimedFn>, usize) {
        match *time {
            TimeSched::Absolute(t) => (self.list, t),
            TimeSched::ContextAbsolute(t) => {
                if let Some(l) = &mut self.context_list {
                    (l, t)
                } else {
                    (self.list, t)
                }
            }
            TimeSched::Relative(t) => (self.list, add_clamped(self.base_tick, t)),
            TimeSched::ContextRelative(t) => {
                if let Some(l) = &mut self.context_list {
                    (l, add_clamped(self.context_tick, t))
                } else {
                    (self.list, add_clamped(self.base_tick, t))
                }
            }
        }
    }
}

impl<'a> SchedContext for Context<'a> {
    fn base_tick(&self) -> usize {
        self.base_tick
    }
    fn context_tick(&self) -> usize {
        self.context_tick
    }
    fn base_tick_period_micros(&self) -> f32 {
        self.base_tick_period_micros
    }
    fn context_tick_period_micros(&self) -> f32 {
        self.context_tick_period_micros
    }
    fn trigger(&mut self, _time: TimeSched, index: usize) {
        let t = self.base_tick; //XXX use actual tick
        let _ = self.trigger_sender.try_send((t, index));
    }
    fn schedule(&mut self, time: TimeSched, func: SchedFn) {
        match self.pop_node() {
            Some(mut n) => {
                n.func = Some(func);
                let (l, t) = self.list_and_tick(&time);
                n.time = t;
                l.insert(n, |n, o| n.time <= o.time);
            }
            None => {
                println!("OOPS");
            }
        }
    }
}

impl SrcSinkUpdater {
    pub fn new(
        node_cache_updater: SyncSender<SchedFnNode>,
        dispose_schedule_receiver: Receiver<Box<dyn Send>>,
    ) -> Self {
        Self {
            node_cache_updater,
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
            break;
        }
        true
    }
}

impl SrcSink {
    pub fn new() -> Self {
        let (dispose_schedule_sender, dispose_schedule_receiver) = sync_channel(1024);
        let (node_cache_updater, node_cache) = sync_channel(1024);
        Self {
            node_cache,
            dispose_schedule_sender,
            updater: Some(SrcSinkUpdater::new(
                node_cache_updater,
                dispose_schedule_receiver,
            )),
        }
    }

    pub fn updater(&mut self) -> Option<SrcSinkUpdater> {
        self.updater.take()
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
        self.list.insert(node, |n, o| n.time <= o.time);
    }

    pub fn pop_node(&mut self) -> Option<SchedFnNode> {
        self.node_cache.try_recv().ok()
    }

    pub fn dispose(&mut self, item: Box<Send>) {
        let _ = self.dispose_schedule_sender.send(item);
    }

    fn eval_triggers(&mut self) {
        while let Some((t, i)) = self.trigger_receiver.try_recv().ok() {
            println!("trigger {} at {}", i, t);
        }
    }

    pub fn run(&mut self, ticks: usize, ticks_per_second: usize) {
        let now = self.time.load(Ordering::SeqCst);
        let next = now + ticks;

        //grab new nodes
        while let Ok(n) = self.schedule_receiver.try_recv() {
            self.add_node(n);
        }

        while let Some(mut timedfn) = self.list.pop_front_while(|n| n.time < next) {
            let current = std::cmp::max(timedfn.time, now); //clamp to now at minimum
            let mut context = Context::new_root(
                current,
                ticks_per_second,
                &mut self.list,
                &mut self.node_cache,
                &mut self.trigger_sender,
            );
            match timedfn.sched_call(&mut context) {
                TimeResched::Relative(time) | TimeResched::ContextRelative(time) => {
                    timedfn.time = current + std::cmp::max(1, time); //schedule minimum of 1 from current
                    self.add_node(timedfn);
                }
                TimeResched::None => {
                    self.dispose(timedfn);
                }
            }
        }
        self.time.store(next, Ordering::SeqCst);
        self.eval_triggers();
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

fn add_clamped(u: usize, i: isize) -> usize {
    if i > 0 {
        u.saturating_add(i as usize)
    } else {
        u.saturating_sub((-i) as usize)
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
        match self.pop_node() {
            Some(mut n) => {
                n.time = add_atomic_time(&self.time, &time); //XXX should we clamp above current time?
                n.func = Some(func);
                self.list.insert(n, |n, o| n.time <= o.time);
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
    use std::thread;

    #[test]
    fn can_vec() {
        let _x: Vec<TimedFn> = (0..20).map({ |_| TimedFn::default() }).collect();
    }

    #[test]
    fn value_set_binding() {
        let pb = Arc::new(SpinlockParamBinding::new(23));
        assert_eq!(23, pb.get());

        let vsb = SpinlockValueSetBinding::new(pb.clone(), 2084);

        //doesn't change it immediately
        assert_eq!(23, pb.get());

        vsb.store();
        assert_eq!(2084, pb.get());

        pb.set(1);
        assert_eq!(1, pb.get());

        vsb.store();
        assert_eq!(2084, pb.get());
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
                        context.trigger(TimeSched::Relative(0), 1);
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
