#[doc(hidden)]
pub extern crate xnor_llist;

use xnor_llist::{List, Node};

use std::thread;
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
pub type SchedFn<'a, Cache> = Box<SchedCall<'a, Cache> + 'a>;

//an object that can schedule SchedFn's and provide a Cache with the cache() method
pub trait Sched<'a, Cache> {
    fn schedule(&mut self, t: TimeSched, func: SchedFn<'a, Cache>);
}

pub trait ExecSched<'a, Cache>: Sched<'a, Cache> {
    fn cache(&mut self) -> &mut Cache;
}

pub trait SchedCall<'a, Cache>: Send {
    fn sched_call(&mut self, sched: &mut ExecSched<'a, Cache>) -> TimeResched;
}

pub trait NodeCache<'a, Cache> {
    fn pop_node(&mut self) -> Option<SchedFnNode<'a, Cache>>;
}

pub trait CacheUpdate<'a, Cache> {
    fn cache(&mut self) -> Option<Cache>;
    fn update(&mut self);
}

//implement sched_call for any Fn that with the correct sig
impl<'a, F: Fn(&mut ExecSched<'a, Cache>) -> TimeResched, Cache> SchedCall<'a, Cache> for F
where
    F: Send,
{
    fn sched_call(&mut self, s: &mut ExecSched<'a, Cache>) -> TimeResched {
        (*self)(s)
    }
}

pub struct TimedFn<'a, Cache> {
    time: UTimePoint,
    func: Option<SchedFn<'a, Cache>>,
}
pub type SchedFnNode<'a, Cache> = Box<xnor_llist::Node<TimedFn<'a, Cache>>>;

impl<'a, Cache> Default for TimedFn<'a, Cache> {
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

pub struct Executor<'a, Cache>
where
    Cache: NodeCache<'a, Cache> + Default,
{
    list: List<TimedFn<'a, Cache>>,
    time: UTimePoint,
    receiver: Receiver<SchedFnNode<'a, Cache>>,
    cache: Cache,
    dispose_sender: SyncSender<Box<Send>>,
}

pub struct Scheduler<'a, CacheUpdater, Cache>
where
    CacheUpdater: CacheUpdate<'a, Cache> + Default,
    Cache: NodeCache<'a, Cache> + Default,
{
    cache_updater: CacheUpdater,
    executor: Option<Executor<'a, Cache>>,
    sender: SyncSender<SchedFnNode<'a, Cache>>,
    dispose_receiver: Option<Receiver<Box<Send>>>,
    dispose_handle: Option<thread::JoinHandle<()>>,
    cache_handle: Option<thread::JoinHandle<()>>,
}

impl<'a, CacheUpdater, Cache> Scheduler<'a, CacheUpdater, Cache>
where
    CacheUpdater: CacheUpdate<'a, Cache> + Default,
    Cache: NodeCache<'a, Cache> + Default,
{
    fn new() -> Self {
        let (sender, receiver) = sync_channel(1024);
        let (dispose_sender, dispose_receiver) = sync_channel(1024);
        let mut updater = CacheUpdater::default();
        Scheduler {
            executor: Some(Executor {
                list: List::new(),
                time: 0,
                receiver,
                cache: updater.cache().unwrap(),
                dispose_sender,
            }),
            sender,
            cache_updater: updater,
            dispose_receiver: Some(dispose_receiver),
            dispose_handle: None,
            cache_handle: None,
        }
    }

    fn executor(&mut self) -> Option<Executor<'a, Cache>> {
        self.executor.take()
    }
}

impl<'a, Cache> Executor<'a, Cache>
where
    Cache: NodeCache<'a, Cache> + Default,
{
    pub fn run(&mut self, ticks: UTimePoint) {
        let next = (self.time + ticks) as ITimePoint;
        //grab new nodes
        while let Ok(n) = self.receiver.try_recv() {
            self.list.insert(n, |n, o| n.time <= o.time);
        }

        let mut reschedule = List::new();
        while let Some(mut timedfn) = self.list.pop_front_while(|n| (n.time as ITimePoint) < next) {
            match timedfn.sched_call(self) {
                TimeResched::Relative(time) => {
                    //XXX manipulate time
                    reschedule.push_back(timedfn);
                }
                TimeResched::ContextRelative(time) => {
                    //XXX manipulate time
                    reschedule.push_back(timedfn);
                }
                TimeResched::None => {
                    //XXX
                    /*
                    if let Err(_) = self.dispose_sender.try_send(timedfn) {
                        println!("XXX how to note this error??");
                    }
                    */
                }
            }
        }
        for n in reschedule.into_iter() {
            self.list.insert(n, |n, o| n.time <= o.time);
        }
        self.time = next as UTimePoint;
    }
}

impl<'a, Cache> SchedCall<'a, Cache> for TimedFn<'a, Cache> {
    fn sched_call(&mut self, s: &mut ExecSched<'a, Cache>) -> TimeResched {
        if let Some(ref mut f) = self.func {
            f.sched_call(s)
        } else {
            TimeResched::None
        }
    }
}

/*

impl SeqSender {
    /// Spawn the helper threads
    pub fn spawn_helper_threads(&mut self) -> () {
        self.spawn_dispose_thread();
        self.spawn_cache_thread();
    }

    /// Spawn a thread that will handle the disposing of boxed items pushed from the schedule
    /// thread
    pub fn spawn_dispose_thread(&mut self) -> () {
        if self.dispose_handle.is_some() {
            return;
        }
        if self.dispose_receiver.is_none() {
            panic!("no dispose receiver!!");
        }

        let mut receiver = None;
        std::mem::swap(&mut receiver, &mut self.dispose_receiver);
        self.dispose_handle = Some(thread::spawn(move || {
            let receiver = receiver.unwrap();
            loop {
                let r = receiver.recv();
                match r {
                    Err(_) => {
                        println!("ditching dispose thread");
                        break;
                    }
                    Ok(_) => println!("got dispose {:?}", thread::current().id()),
                }
            }
        }));
    }

    /// Spawn a thread to fill up the node cache so we can schedule in the schedule thread
    pub fn spawn_cache_thread(&mut self) -> () {
        if self.cache_handle.is_some() {
            return;
        }
        if self.node_cache_sender.is_none() {
            panic!("no cache sender");
        }

        let mut sender = None;
        std::mem::swap(&mut sender, &mut self.node_cache_sender);
        self.cache_handle = Some(thread::spawn(move || {
            let sender = sender.unwrap();
            loop {
                let r = sender.send(Node::new_boxed(TimedFn {
                    func: None,
                    time: 0,
                }));
                match r {
                    Err(_) => {
                        println!("ditching cache thread");
                        break;
                    }
                    Ok(_) => (),
                }
            }
        }));
    }
}
*/

impl<'a, CacheUpdater, Cache> Sched<'a, Cache> for Scheduler<'a, CacheUpdater, Cache>
where
    CacheUpdater: CacheUpdate<'a, Cache> + Default,
    Cache: NodeCache<'a, Cache> + Default,
{
    fn schedule(&mut self, _time: TimeSched, func: SchedFn<'a, Cache>) {
        let t: usize = 0; //XXX translate from time
        let f = Node::new_boxed(TimedFn {
            func: Some(func),
            time: t,
        });
        self.sender.send(f).unwrap();
    }
}

impl<'a, Cache> Sched<'a, Cache> for Executor<'a, Cache>
where
    Cache: NodeCache<'a, Cache> + Default,
{
    fn schedule(&mut self, _time: TimeSched, func: SchedFn<'a, Cache>) {
        let t: usize = 0; //XXX translate from time
        match self.cache.pop_node() {
            Some(mut n) => {
                n.time = t;
                n.func = Some(func);
                self.list.insert(n, |n, o| n.time <= o.time);
            }
            None => {
                println!("OOPS");
            }
        }
    }
}

impl<'a, Cache> ExecSched<'a, Cache> for Executor<'a, Cache>
where
    Cache: NodeCache<'a, Cache> + Default,
{
    fn cache(&mut self) -> &mut Cache {
        &mut self.cache
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let x: Vec<TimedFn<()>> = (0..20).map({ |_| TimedFn::default() }).collect();
    }

    impl<'a> NodeCache<'a, ()> for () {
        fn pop_node(&mut self) -> Option<SchedFnNode<'a, ()>> {
            Some(Node::new_boxed(Default::default()))
        }
    }

    impl<'a> CacheUpdate<'a, ()> for () {
        fn cache(&mut self) -> Option<()> {
            Some(())
        }
        fn update(&mut self) {}
    }

    #[test]
    fn scheduler() {
        type SImpl<'a> = Scheduler<'a, (), ()>;
        let mut s = SImpl::new();
        let e = s.executor();
        assert!(e.is_some());
        s.schedule(
            TimeSched::Absolute(0),
            Box::new(move |_s: &mut ExecSched<()>| {
                println!("Closure in schedule");
                TimeResched::Relative(3)
            }),
        );

        let mut e = e.unwrap();
        e.run(32);
        e.run(32);
    }
}
