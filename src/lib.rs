#[doc(hidden)]
pub extern crate xnor_llist;

pub use xnor_llist::{List, Node};

use std::thread;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};

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
pub type SchedFn<Cache, Sink, Context> = Box<SchedCall<Cache, Sink, Context>>;

//an object that can schedule SchedFn's and provide a Cache with the cache() method
pub trait Sched<Cache, Sink, Context> {
    fn schedule(&mut self, t: TimeSched, func: SchedFn<Cache, Sink, Context>);
}

pub trait ExecSched<Cache, Sink, Context>: Sched<Cache, Sink, Context> {
    fn cache(&mut self) -> &mut Cache;
    fn sink(&mut self) -> &mut Sink;
}

pub trait SchedCall<Cache, Sink, Context>: Send {
    fn sched_call(&mut self, sched: &mut ExecSched<Cache, Sink, Context>) -> TimeResched;
}

pub trait NodeCache<Cache, Sink, Context> {
    fn pop_node(&mut self) -> Option<SchedFnNode<Cache, Sink, Context>>;
}

pub trait CacheUpdate: Send {
    fn update(&mut self) -> bool;
}

pub trait CacheCreate<Cache, Update: CacheUpdate> {
    fn cache(&mut self) -> Option<Cache>;
    fn updater(&mut self) -> Option<Update>;
}

//implement sched_call for any Fn that with the correct sig
impl<
    F: Fn(&mut ExecSched<Cache, Sink, Context>) -> TimeResched,
    Cache,
    Sink,
    Context,
> SchedCall<Cache, Sink, Context> for F
where
    F: Send,
{
    fn sched_call(&mut self, s: &mut ExecSched<Cache, Sink, Context>) -> TimeResched {
        (*self)(s)
    }
}

pub struct TimedFn<Cache, Sink, Context> {
    time: usize,
    func: Option<SchedFn<Cache, Sink, Context>>,
}
pub type SchedFnNode<Cache, Sink, Context> = Box<xnor_llist::Node<TimedFn<Cache, Sink, Context>>>;

impl<Cache, Sink, Context> Default for TimedFn<Cache, Sink, Context> {
    fn default() -> Self {
        TimedFn {
            time: 0,
            func: None,
        }
    }
}

pub struct Executor<Cache, Sink, Context>
where
    Cache: NodeCache<Cache, Sink, Context>,
    Sink: Default + 'static,
{
    list: List<TimedFn<Cache, Sink, Context>>,
    time: Arc<AtomicUsize>,
    receiver: Receiver<SchedFnNode<Cache, Sink, Context>>,
    cache: Cache,
    sink: Sink,
    dispose_sender: SyncSender<Box<Send>>,
}

pub struct Scheduler<CacheCreator, Cache, Sink, Context, Update>
where
    CacheCreator: CacheCreate<Cache, Update> + Default,
    Cache: NodeCache<Cache, Sink, Context>,
    Update: CacheUpdate + 'static,
    Sink: Default + 'static,
{
    time: Arc<AtomicUsize>,
    cache: CacheCreator,
    executor: Option<Executor<Cache, Sink, Context>>,
    sender: SyncSender<SchedFnNode<Cache, Sink, Context>>,
    dispose_receiver: Option<Receiver<Box<Send>>>,
    dispose_handle: Option<thread::JoinHandle<()>>,
    cache_handle: Option<thread::JoinHandle<()>>,
    phantom_update: std::marker::PhantomData<Update>,
}

impl<CacheCreator, Cache, Sink, Context, Update>
    Scheduler<CacheCreator, Cache, Sink, Context, Update>
where
    CacheCreator: CacheCreate<Cache, Update> + Default,
    Cache: NodeCache<Cache, Sink, Context>,
    Update: CacheUpdate + 'static,
    Sink: Default + 'static,
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
                sink: Default::default(),
                cache: cache.cache().unwrap(),
                dispose_sender,
            }),
            sender,
            cache: cache,
            dispose_receiver: Some(dispose_receiver),
            dispose_handle: None,
            cache_handle: None,
            phantom_update: std::marker::PhantomData,
        }
    }

    pub fn executor(&mut self) -> Option<Executor<Cache, Sink, Context>> {
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

impl<Cache, Sink, Context: 'static> Executor<Cache, Sink, Context>
where
    Cache: NodeCache<Cache, Sink, Context> + 'static,
    Sink: Default + 'static,
{
    fn add_node(&mut self, node: SchedFnNode<Cache, Sink, Context>) {
        self.list.insert(node, |n, o| n.time <= o.time);
    }

    pub fn run(&mut self, ticks: usize) {
        let next = self.time.load(Ordering::SeqCst) + ticks;

        //grab new nodes
        while let Ok(n) = self.receiver.try_recv() {
            self.add_node(n);
        }

        let mut reschedule = List::new();
        while let Some(mut timedfn) = self.list.pop_front_while(|n| n.time < next) {
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
            self.add_node(n);
        }
        self.time.store(next, Ordering::SeqCst);
    }
}

impl<Cache, Sink, Context> SchedCall<Cache, Sink, Context> for TimedFn<Cache, Sink, Context> {
    fn sched_call(&mut self, s: &mut ExecSched<Cache, Sink, Context>) -> TimeResched {
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

impl<CacheCreator, Cache, Sink, Context, Update> Sched<Cache, Sink, Context>
    for Scheduler<CacheCreator, Cache, Sink, Context, Update>
where
    CacheCreator: CacheCreate<Cache, Update> + Default,
    Cache: NodeCache<Cache, Sink, Context>,
    Update: CacheUpdate + 'static,
    Sink: Default + 'static,
{
    fn schedule(&mut self, time: TimeSched, func: SchedFn<Cache, Sink, Context>) {
        let f = Node::new_boxed(TimedFn {
            func: Some(func),
            time: add_time(&self.time, &time),
        });
        self.sender.send(f).unwrap();
    }
}

impl<Cache, Sink, Context> Sched<Cache, Sink, Context> for Executor<Cache, Sink, Context>
where
    Cache: NodeCache<Cache, Sink, Context>,
    Sink: Default + 'static,
{
    fn schedule(&mut self, time: TimeSched, func: SchedFn<Cache, Sink, Context>) {
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

impl<Cache, Sink, Context> ExecSched<Cache, Sink, Context> for Executor<Cache, Sink, Context>
where
    Cache: NodeCache<Cache, Sink, Context>,
    Sink: Default + 'static,
{
    fn cache(&mut self) -> &mut Cache {
        &mut self.cache
    }

    fn sink(&mut self) -> &mut Sink {
        &mut self.sink
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn can_vec() {
        let _x: Vec<TimedFn<(), (), ()>> = (0..20).map({ |_| TimedFn::default() }).collect();
    }

    impl NodeCache<(), (), ()> for () {
        fn pop_node(&mut self) -> Option<SchedFnNode<(), (), ()>> {
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
        type SImpl = Scheduler<(), (), (), (), ()>;
        type EImpl<'a> = ExecSched<(), (), ()> + 'a;
        let mut s = SImpl::new();
        s.spawn_helper_threads();

        let e = s.executor();
        assert!(e.is_some());
        s.schedule(
            TimeSched::Absolute(0),
            Box::new(move |s: &mut EImpl| {
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
