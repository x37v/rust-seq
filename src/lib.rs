#[doc(hidden)]
pub extern crate xnor_llist;

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
}

//an object to be put into a schedule and called later
pub type SchedFn<'a, Cache> = Box<SchedCall<Cache> + 'a>;

//an object that can schedule SchedFn's and provide a Cache with the cache() method
pub trait Sched<Cache> {
    fn schedule(&mut self, t: TimeSched, n: SchedFn<Cache>);
    fn cache(&mut self) -> &mut Cache;
}

pub trait SchedCall<Cache>: Send {
    fn sched_call(&mut self, sched: &mut Sched<Cache>) -> TimeResched;
}

pub trait NodeCache<'a, Cache> {
    fn pop_node(&mut self) -> Option<SchedFnNode<'a, Cache>>;
}

//implement sched_call for any Fn that with the correct sig
impl<F: Fn(&mut Sched<Cache>) -> TimeResched, Cache> SchedCall<Cache> for F
where
    F: Send,
{
    fn sched_call(&mut self, s: &mut Sched<Cache>) -> TimeResched {
        (*self)(s)
    }
}

pub struct TimedFn<'a, Cache> {
    time: ITimePoint,
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
    list: xnor_llist::List<TimedFn<'a, Cache>>,
    receiver: Receiver<SchedFnNode<'a, Cache>>,
    cache: Cache,
    dispose_sender: SyncSender<Box<Send>>,
}

pub struct Scheduler<'a, Cache>
where
    Cache: NodeCache<'a, Cache> + Default,
{
    executor: Option<Executor<'a, Cache>>,
    sender: SyncSender<SchedFnNode<'a, Cache>>,
    dispose_receiver: Option<Receiver<Box<Send>>>,
    dispose_handle: Option<thread::JoinHandle<()>>,
    cache_handle: Option<thread::JoinHandle<()>>,
}

impl<'a, Cache> Scheduler<'a, Cache>
where
    Cache: NodeCache<'a, Cache> + Default,
{
    fn new() -> Self {
        let (sender, receiver) = sync_channel(1024);
        let (dispose_sender, dispose_receiver) = sync_channel(1024);
        Scheduler {
            executor: None,
            sender,
            dispose_receiver: Some(dispose_receiver),
            dispose_handle: None,
            cache_handle: None,
        }
    }
}

/*

impl<Context, Cache, Sink> SchedCall<Context, Cache, Sink> for TimedFn<Context, Cache, Sink> {
    fn sched_call<S>(&mut self, s: &mut S) -> Option<UTimePoint>
    where
        S: Sched<Context, Cache, Sink>,
    {
        if let Some(ref mut f) = self.func {
            f.sched_call(s)
        } else {
            None
        }
    }
}

pub trait SeqCached<T> {
    fn pop() -> Option<Box<T>>;
    fn push(v: Box<T>) -> ();
}

pub trait CacheUpdate {
    fn update(&mut self);
}

pub struct SeqSender {
    sender: SyncSender<SeqFnNode>,
    node_cache_sender: Option<SyncSender<SeqFnNode>>,
    dispose_receiver: Option<Receiver<Box<Send>>>,
    dispose_handle: Option<thread::JoinHandle<()>>,
    cache_handle: Option<thread::JoinHandle<()>>,
}

pub struct SeqExecuter {
    list: xnor_llist::List<TimedFn>,
    receiver: Receiver<SeqFnNode>,
    node_cache_receiver: Receiver<SeqFnNode>,
    dispose_sender: SyncSender<Box<Send>>,
    time: UTimePoint,
}

pub fn sequencer() -> (SeqSender, SeqExecuter) {
    let (sender, receiver) = sync_channel(1024);
    let (node_cache_sender, node_cache_receiver) = sync_channel(1024);
    let (dispose_sender, dispose_receiver) = sync_channel(1024);
    (
        SeqSender {
            sender,
            node_cache_sender: Some(node_cache_sender),
            dispose_receiver: Some(dispose_receiver),
            dispose_handle: None,
            cache_handle: None,
        },
        SeqExecuter {
            receiver,
            node_cache_receiver,
            dispose_sender,
            list: xnor_llist::List::new(),
            time: 0,
        },
    )
}

impl Sched for SeqSender {
    fn schedule(&mut self, time: ITimePoint, func: SeqFn) {
        let f = xnor_llist::Node::new_boxed(TimedFn {
            func: Some(func),
            time,
        });
        self.sender.send(f).unwrap();
    }
}

impl Sched for SeqExecuter {
    fn schedule(&mut self, time: ITimePoint, func: SeqFn) {
        match self.node_cache_receiver.try_recv() {
            Ok(mut n) => {
                n.time = time;
                n.func = Some(func);
                self.list.insert(n, |n, o| n.time <= o.time);
            }
            Err(_) => {
                println!("OOPS");
            }
        }
    }
}

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
                let r = sender.send(xnor_llist::Node::new_boxed(TimedFn {
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

impl SeqExecuter {
    pub fn time(&self) -> UTimePoint {
        self.time
    }

    pub fn run(&mut self, ticks: UTimePoint) {
        let next = (self.time + ticks) as ITimePoint;
        //grab new nodes
        while let Ok(n) = self.receiver.try_recv() {
            self.list.insert(n, |n, o| n.time <= o.time);
        }

        let mut reschedule = xnor_llist::List::new();
        while let Some(mut timedfn) = self.list.pop_front_while(|n| n.time < next) {
            if let Some(t) = timedfn.sched_call(self) {
                timedfn.time = t as ITimePoint + timedfn.time;
                //XXX clamp bottom to next?
                reschedule.push_back(timedfn);
            } else {
                if let Err(_) = self.dispose_sender.try_send(timedfn) {
                    println!("XXX how to note this error??");
                }
            }
        }
        for n in reschedule.into_iter() {
            self.list.insert(n, |n, o| n.time <= o.time);
        }
        self.time = next as UTimePoint;
    }
}
*/

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let x: Vec<TimedFn<()>> = (0..20).map({ |_| TimedFn::default() }).collect();
    }

    impl<'a> NodeCache<'a, ()> for () {
        fn pop_node(&mut self) -> Option<SchedFnNode<'a, ()>> {
            None
        }
    }

    #[test]
    fn scheduler() {
        type Impl<'a> = Scheduler<'a, ()>;
        let s = Impl::new();
    }
}
