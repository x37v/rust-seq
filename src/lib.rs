#[doc(hidden)]
pub extern crate xnor_llist;

use std::thread;
use std::sync::Arc;
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};

pub type ITimePoint = isize;
pub type UTimePoint = usize;
pub type SeqFn = Arc<SchedCall>;

pub struct TimedFn {
    time: ITimePoint,
    func: SeqFn,
}

impl TimedFn {
    pub fn new(func: SeqFn) -> Self {
        TimedFn { func, time: 0 }
    }
}

pub type SeqFnNode = Box<xnor_llist::Node<TimedFn>>;

#[macro_export]
macro_rules! boxed_fn {
    ($x:expr) => {
        $crate::xnor_llist::Node::new_boxed($crate::TimedFn::new(Arc::new($x)))
    }
}

//XXX is it cool to just say that these shits are Sync and send?
pub trait SchedCall: Sync + Send {
    fn sched_call(&mut self, &mut Sched) -> Option<UTimePoint>;
}

pub trait Sched {
    fn schedule(&mut self, t: ITimePoint, n: SeqFnNode);
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
        if let Some(f) = Arc::get_mut(&mut self.func) {
            f.sched_call(s)
        } else {
            None
        }
    }
}

pub trait SeqCached<T> {
    fn pop() -> Option<Arc<T>>;
    fn push(v: Arc<T>) -> ();
}

pub struct SeqSender {
    sender: SyncSender<SeqFnNode>,
    dispose_receiver: Option<Receiver<SeqFnNode>>,
    dispose_handle: Option<thread::JoinHandle<()>>,
}

pub struct SeqExecuter {
    list: xnor_llist::List<TimedFn>,
    receiver: Receiver<SeqFnNode>,
    dispose_sender: SyncSender<SeqFnNode>,
    time: UTimePoint,
}

pub fn sequencer() -> (SeqSender, SeqExecuter) {
    let (sender, receiver) = sync_channel(1024);
    let (dispose_sender, dispose_receiver) = sync_channel(1024);
    (
        SeqSender {
            sender,
            dispose_receiver: Some(dispose_receiver),
            dispose_handle: None,
        },
        SeqExecuter {
            receiver,
            dispose_sender,
            list: xnor_llist::List::new(),
            time: 0,
        },
    )
}

impl Sched for SeqSender {
    fn schedule(&mut self, t: ITimePoint, mut f: SeqFnNode) {
        f.time = t;
        self.sender.send(f).unwrap();
    }
}

impl Sched for SeqExecuter {
    fn schedule(&mut self, t: ITimePoint, mut f: SeqFnNode) {
        f.time = t;
        self.list.insert(f, |n, o| n.time <= o.time);
    }
}

impl SeqSender {
    /// Spawn a thread that will handle the disposing of nodes
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
                    Ok(_) => println!("got dispose"),
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
            self.schedule(n.time, n);
        }
        self.time = next as UTimePoint;
    }
}
