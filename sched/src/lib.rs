#![feature(nll)]

#[doc(hidden)]
pub extern crate xnor_llist;

pub use xnor_llist::List as LList;
pub use xnor_llist::Node as LNode;

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

pub struct Executor {
    list: LList<TimedFn>,
    time: Arc<AtomicUsize>,
    schedule_receiver: Receiver<SchedFnNode>,
    node_cache: Receiver<SchedFnNode>,
    dispose_schedule_sender: SyncSender<Box<dyn Send>>,
}

pub struct Scheduler {
    time: Arc<AtomicUsize>,
    executor: Option<Executor>,
    schedule_sender: SyncSender<SchedFnNode>,
    node_cache_updater: Option<SyncSender<SchedFnNode>>,
    dispose_schedule_receiver: Option<Receiver<Box<dyn Send>>>,
    helper_handle: Option<thread::JoinHandle<()>>,
}

pub struct Context<'a> {
    base_tick: usize,
    context_tick: usize,
    base_tick_period_micros: f32,
    context_tick_period_micros: f32,
    list: &'a LList<TimedFn>,
}

impl<'a> Context<'a> {
    fn new_root(tick: usize, ticks_per_second: usize, list: &'a mut LList<TimedFn>) -> Self {
        let tpm = 1e6f32 / (ticks_per_second as f32);
        Context {
            base_tick: tick,
            context_tick: tick,
            base_tick_period_micros: tpm,
            context_tick_period_micros: tpm,
            list,
        }
    }
}

impl SchedContext for Context {
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
    fn trigger(&mut self, _time: TimeSched, _index: usize) {}
    fn schedule(&mut self, _t: TimeSched, _func: SchedFn) {}
}

impl Scheduler {
    pub fn new() -> Self {
        let (schedule_sender, schedule_receiver) = sync_channel(1024);
        let (dispose_schedule_sender, dispose_schedule_receiver) = sync_channel(1024);
        let (node_cache_updater, node_cache) = sync_channel(1024);
        let time = Arc::new(AtomicUsize::new(0));
        Scheduler {
            time: time.clone(),
            executor: Some(Executor {
                list: LList::new(),
                time,
                schedule_receiver,
                dispose_schedule_sender,
                node_cache,
            }),
            schedule_sender,
            dispose_schedule_receiver: Some(dispose_schedule_receiver),
            node_cache_updater: Some(node_cache_updater),
            helper_handle: None,
        }
    }

    pub fn spawn_helper_threads(&mut self) {
        let dispose_schedule_receiver = self.dispose_schedule_receiver.take().unwrap();
        let node_cache_updater = self.node_cache_updater.take().unwrap();
        self.helper_handle = Some(thread::spawn(move || {
            let sleep_time = std::time::Duration::from_millis(5);
            loop {
                if let Err(TryRecvError::Disconnected) = dispose_schedule_receiver.try_recv() {
                    break;
                }
                if let Err(TrySendError::Disconnected(_)) =
                    node_cache_updater.try_send(LNode::new_boxed(Default::default()))
                {
                    break;
                }
                thread::sleep(sleep_time);
            }
        }));
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

    pub fn run(&mut self, ticks: usize, ticks_per_second: usize) {
        let now = self.time.load(Ordering::SeqCst);
        let next = now + ticks;

        //grab new nodes
        while let Ok(n) = self.schedule_receiver.try_recv() {
            self.add_node(n);
        }

        while let Some(mut timedfn) = self.list.pop_front_while(|n| n.time < next) {
            let current = std::cmp::max(timedfn.time, now); //clamp to now at minimum
            let mut context = Context::new_root(current, ticks_per_second);
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

fn add_time(current: &Arc<AtomicUsize>, time: &TimeSched) -> usize {
    match *time {
        TimeSched::Absolute(t) | TimeSched::ContextAbsolute(t) => t,
        TimeSched::Relative(t) | TimeSched::ContextRelative(t) => {
            add_clamped(current.load(Ordering::SeqCst), t)
        }
    }
}

impl Sched for Scheduler {
    fn schedule(&mut self, time: TimeSched, func: SchedFn) {
        let f = LNode::new_boxed(TimedFn {
            func: Some(func),
            time: add_time(&self.time, &time),
        });
        self.schedule_sender.send(f).unwrap();
    }
}

impl Sched for Executor {
    fn schedule(&mut self, time: TimeSched, func: SchedFn) {
        match self.pop_node() {
            Some(mut n) => {
                n.time = add_time(&self.time, &time); //XXX should we clamp above current time?
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
