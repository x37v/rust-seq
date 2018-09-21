#[doc(hidden)]
pub extern crate xnor_llist;

pub use xnor_llist::{List, Node};

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::sync::Arc;

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
pub type SchedFnNode = Box<xnor_llist::Node<TimedFn>>;

impl Default for TimedFn {
    fn default() -> Self {
        TimedFn {
            time: 0,
            func: None,
        }
    }
}

pub struct Executor {
    list: List<TimedFn>,
    time: Arc<AtomicUsize>,
    receiver: Receiver<SchedFnNode>,
    node_cache: Receiver<SchedFnNode>,
    dispose_sender: SyncSender<Box<dyn Send>>,
}

pub struct Scheduler {
    time: Arc<AtomicUsize>,
    executor: Option<Executor>,
    sender: SyncSender<SchedFnNode>,
    node_cache_updater: SyncSender<SchedFnNode>,
    dispose_receiver: Receiver<Box<dyn Send>>,
}

pub struct Context {
    base_tick: usize,
    context_tick: usize,
    base_tick_period_micros: f32,
    context_tick_period_micros: f32,
}

impl Context {
    fn new_root(tick: usize, ticks_per_second: usize) -> Self {
        let tpm = 1e-6f32 / (ticks_per_second as f32);
        Context {
            base_tick: tick,
            context_tick: tick,
            base_tick_period_micros: tpm,
            context_tick_period_micros: tpm,
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
        let (sender, receiver) = sync_channel(1024);
        let (dispose_sender, dispose_receiver) = sync_channel(1024);
        let (node_cache_updater, node_cache) = sync_channel(1024);
        let time = Arc::new(AtomicUsize::new(0));
        Scheduler {
            time: time.clone(),
            executor: Some(Executor {
                list: List::new(),
                time,
                receiver,
                dispose_sender,
                node_cache,
            }),
            sender,
            dispose_receiver,
            node_cache_updater,
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
        let _ = self.dispose_sender.send(item);
    }

    pub fn run(&mut self, ticks: usize, _ticks_per_second: usize) {
        let now = self.time.load(Ordering::SeqCst);
        let next = now + ticks;

        //grab new nodes
        while let Ok(n) = self.receiver.try_recv() {
            self.add_node(n);
        }

        while let Some(mut timedfn) = self.list.pop_front_while(|n| n.time < next) {
            let current = std::cmp::max(timedfn.time, now); //clamp to now at minimum
            let mut context = Context::new_root(0, 0);
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
        let f = Node::new_boxed(TimedFn {
            func: Some(func),
            time: add_time(&self.time, &time),
        });
        self.sender.send(f).unwrap();
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

/*
#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn can_vec() {
        let _x: Vec<TimedFn<(), ()>> = (0..20).map({ |_| TimedFn::default() }).collect();
    }

    impl NodeSrc<(), ()> for () {
        fn pop_node(&mut self) -> Option<SchedFnNode<(), ()>> {
            Some(Node::new_boxed(Default::default()))
        }
    }

    impl DisposeSink for () {
        fn dispose(&mut self, _item: Box<Send>) {
            //drop
        }
    }

    impl SrcSnkUpdate for () {
        fn update(&mut self) -> bool {
            true
        }
    }

    impl ContextBase for () {
        fn from_root(_tick: usize, _ticks_per_second: usize) -> Self {
            ()
        }

        fn tick(&self) -> usize {
            0
        }
        fn ticks_per_second(&self) -> Option<usize> {
            None
        }
    }

    impl SrcSnkCreate<(), ()> for () {
        fn src_sink(&mut self) -> Option<()> {
            Some(())
        }
        fn updater(&mut self) -> Option<()> {
            Some(())
        }
    }

    #[test]
    fn fake_src_sink() {
        type SImpl = Scheduler<(), (), (), ()>;
        type EImpl<'a> = ExecSched<(), ()> + 'a;
        let mut s = SImpl::new();
        s.spawn_helper_threads();

        let e = s.executor();
        assert!(e.is_some());
        s.schedule(
            TimeSched::Absolute(0),
            Box::new(move |s: &mut EImpl, _context: &mut ()| {
                println!("Closure in schedule");
                assert!(s.src_sink().pop_node().is_some());
                assert_eq!(s.context(), ());
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
*/
