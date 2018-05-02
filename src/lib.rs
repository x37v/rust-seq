#[doc(hidden)]
pub extern crate xnor_llist;

pub use xnor_llist::{List, Node};

use std::thread;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};

pub type ITimePoint = isize;
pub type UTimePoint = usize;

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

//an object to be put into a schedule and called later
pub type SchedFn<Cache> = Box<SchedCall<Cache>>;

//an object that can schedule SchedFn's and provide a Cache with the cache() method
pub trait Sched<Cache> {
    fn schedule(&mut self, t: TimeSched, func: SchedFn<Cache>);
}

pub trait ExecSched<Cache>: Sched<Cache> {
    fn cache(&mut self) -> &mut Cache;
}

pub trait SchedCall<Cache>: Send {
    fn sched_call(&mut self, sched: &mut ExecSched<Cache>) -> TimeResched;
}

pub trait NodeCache<Cache> {
    fn pop_node(&mut self) -> Option<SchedFnNode<Cache>>;
}

pub trait CacheUpdate: Send {
    fn update(&mut self) -> bool;
}

pub trait CacheCreate<Cache, Update: CacheUpdate> {
    fn cache(&mut self) -> Option<Cache>;
    fn updater(&mut self) -> Option<Update>;
}

//implement sched_call for any Fn that with the correct sig
impl<F: Fn(&mut ExecSched<Cache>) -> TimeResched, Cache> SchedCall<Cache> for F
where
    F: Send,
{
    fn sched_call(&mut self, s: &mut ExecSched<Cache>) -> TimeResched {
        (*self)(s)
    }
}

pub struct TimedFn<Cache> {
    time: UTimePoint,
    func: Option<SchedFn<Cache>>,
}
pub type SchedFnNode<Cache> = Box<xnor_llist::Node<TimedFn<Cache>>>;

impl<Cache> Default for TimedFn<Cache> {
    fn default() -> Self {
        TimedFn {
            time: 0,
            func: None,
        }
    }
}

#[macro_export]
macro_rules! wrap_fn {
    ($x:expr) => {
        Box::new($x)
    }
}

pub struct Executor<Cache>
where
    Cache: NodeCache<Cache>,
{
    list: List<TimedFn<Cache>>,
    time: Arc<AtomicUsize>,
    receiver: Receiver<SchedFnNode<Cache>>,
    cache: Cache,
    dispose_sender: SyncSender<Box<Send>>,
}

pub struct Scheduler<CacheCreator, Cache, Update>
where
    CacheCreator: CacheCreate<Cache, Update> + Default,
    Cache: NodeCache<Cache>,
    Update: CacheUpdate + 'static,
{
    time: Arc<AtomicUsize>,
    cache: CacheCreator,
    executor: Option<Executor<Cache>>,
    sender: SyncSender<SchedFnNode<Cache>>,
    dispose_receiver: Option<Receiver<Box<Send>>>,
    dispose_handle: Option<thread::JoinHandle<()>>,
    cache_handle: Option<thread::JoinHandle<()>>,
    phantom: std::marker::PhantomData<Update>,
}

impl<CacheCreator, Cache, Update> Scheduler<CacheCreator, Cache, Update>
where
    CacheCreator: CacheCreate<Cache, Update> + Default,
    Cache: NodeCache<Cache>,
    Update: CacheUpdate + 'static,
{
    pub fn new() -> Self {
        let (sender, receiver) = sync_channel(1024);
        let (dispose_sender, dispose_receiver) = sync_channel(1024);
        let mut cache = CacheCreator::default();
        let time = Arc::new(AtomicUsize::new(0));
        Scheduler {
            time: time.clone(),
            executor: Some(Executor {
                list: List::new(),
                time: time,
                receiver,
                cache: cache.cache().unwrap(),
                dispose_sender,
            }),
            sender,
            cache: cache,
            dispose_receiver: Some(dispose_receiver),
            dispose_handle: None,
            cache_handle: None,
            phantom: std::marker::PhantomData,
        }
    }

    pub fn executor(&mut self) -> Option<Executor<Cache>> {
        self.executor.take()
    }

    /// Spawn the helper threads
    pub fn spawn_helper_threads(&mut self) -> () {
        self.spawn_dispose_thread();
        self.spawn_cache_thread();
    }

    /// Spawn a thread that will handle the disposing of boxed items pushed from the execution thread
    pub fn spawn_dispose_thread(&mut self) -> () {
        if self.dispose_handle.is_some() {
            return;
        }
        let receiver = self.dispose_receiver.take().unwrap();
        self.dispose_handle = Some(thread::spawn(move || loop {
            let r = receiver.recv();
            match r {
                Err(_) => {
                    println!("ditching dispose thread");
                    break;
                }
                Ok(_) => println!("got dispose {:?}", thread::current().id()),
            }
        }));
    }

    /// Spawn a thread to fill up the cache so we can get objects in the execution thread
    /// Note: This calls update once in the current thread in order to get the cache full
    /// immediately
    pub fn spawn_cache_thread(&mut self) -> () {
        if self.cache_handle.is_some() {
            return;
        }

        let mut updater = self.cache.updater().unwrap();
        updater.update(); //get an initial update
        self.cache_handle = Some(thread::spawn(move || {
            let sleep_time = std::time::Duration::from_millis(5);
            while updater.update() {
                thread::sleep(sleep_time);
            }
        }));
    }
}

impl<Cache: 'static> Executor<Cache>
where
    Cache: NodeCache<Cache>,
{
    pub fn run(&mut self, ticks: UTimePoint) {
        let next = (self.time.load(Ordering::SeqCst) + ticks) as ITimePoint;
        //grab new nodes
        while let Ok(n) = self.receiver.try_recv() {
            self.list.insert(n, |n, o| n.time <= o.time);
        }

        let mut reschedule = List::new();
        while let Some(mut timedfn) = self.list.pop_front_while(|n| (n.time as ITimePoint) < next) {
            match timedfn.sched_call(self) {
                TimeResched::Relative(time) | TimeResched::ContextRelative(time) => {
                    timedfn.time = timedfn.time + time;
                    reschedule.push_back(timedfn);
                }
                TimeResched::None => {
                    if let Err(_) = self.dispose_sender.try_send(timedfn) {
                        println!("XXX how to note this error??");
                    }
                }
            }
        }
        for n in reschedule.into_iter() {
            self.list.insert(n, |n, o| n.time <= o.time);
        }
        self.time.store(next as usize, Ordering::SeqCst);
    }
}

impl<Cache> SchedCall<Cache> for TimedFn<Cache> {
    fn sched_call(&mut self, s: &mut ExecSched<Cache>) -> TimeResched {
        if let Some(ref mut f) = self.func {
            f.sched_call(s)
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
    match time {
        &TimeSched::Absolute(t) | &TimeSched::ContextAbsolute(t) => t,
        &TimeSched::Relative(t) | &TimeSched::ContextRelative(t) => {
            add_clamped(current.load(Ordering::SeqCst), t)
        }
    }
}

impl<CacheCreator, Cache, Update> Sched<Cache> for Scheduler<CacheCreator, Cache, Update>
where
    CacheCreator: CacheCreate<Cache, Update> + Default,
    Cache: NodeCache<Cache>,
    Update: CacheUpdate + 'static,
{
    fn schedule(&mut self, time: TimeSched, func: SchedFn<Cache>) {
        let f = Node::new_boxed(TimedFn {
            func: Some(func),
            time: add_time(&self.time, &time),
        });
        self.sender.send(f).unwrap();
    }
}

impl<Cache> Sched<Cache> for Executor<Cache>
where
    Cache: NodeCache<Cache>,
{
    fn schedule(&mut self, time: TimeSched, func: SchedFn<Cache>) {
        match self.cache.pop_node() {
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

impl<Cache> ExecSched<Cache> for Executor<Cache>
where
    Cache: NodeCache<Cache>,
{
    fn cache(&mut self) -> &mut Cache {
        &mut self.cache
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn can_vec() {
        let _x: Vec<TimedFn<()>> = (0..20).map({ |_| TimedFn::default() }).collect();
    }

    impl NodeCache<()> for () {
        fn pop_node(&mut self) -> Option<SchedFnNode<()>> {
            Some(Node::new_boxed(Default::default()))
        }
    }

    impl CacheUpdate for () {
        fn update(&mut self) -> bool {
            true
        }
    }

    impl CacheCreate<(), ()> for () {
        fn cache(&mut self) -> Option<()> {
            Some(())
        }
        fn updater(&mut self) -> Option<()> {
            Some(())
        }
    }

    #[test]
    fn fake_cache() {
        type SImpl = Scheduler<(), (), ()>;
        let mut s = SImpl::new();
        s.spawn_helper_threads();

        let e = s.executor();
        assert!(e.is_some());
        s.schedule(
            TimeSched::Absolute(0),
            Box::new(move |s: &mut ExecSched<()>| {
                println!("Closure in schedule");
                assert!(s.cache().pop_node().is_some());
                TimeResched::Relative(3)
            }),
        );

        let child = thread::spawn(move || {
            let mut e = e.unwrap();
            e.run(32);
            e.run(32);
        });

        assert!(child.join().is_ok());
    }
}
