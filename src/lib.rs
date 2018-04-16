#[doc(hidden)]
pub extern crate xnor_llist;

pub mod midi;

use std::thread;
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};

pub type ITimePoint = isize;
pub type UTimePoint = usize;
pub type SeqFn = Box<SchedCall>;

pub struct TimedFn {
    time: ITimePoint,
    func: Option<SeqFn>,
}

pub type SeqFnNode = Box<xnor_llist::Node<TimedFn>>;

#[macro_export]
macro_rules! wrap_fn {
    ($x:expr) => {
        Box::new($x)
    }
}

//XXX is it cool to just say that these shits are Sync and send?
pub trait SchedCall: Sync + Send {
    fn sched_call(&mut self, &mut Sched) -> Option<UTimePoint>;
}

pub trait Sched {
    fn schedule(&mut self, t: ITimePoint, n: SeqFn);
}

impl<F: Fn(&mut Sched) -> Option<UTimePoint>> SchedCall for F
where
    F: Sync + Send,
{
    fn sched_call(&mut self, s: &mut Sched) -> Option<UTimePoint> {
        (*self)(s)
    }
}

impl SchedCall for TimedFn {
    fn sched_call(&mut self, s: &mut Sched) -> Option<UTimePoint> {
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
