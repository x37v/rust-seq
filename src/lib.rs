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

pub trait ContextInit {
    fn with_time(time: usize) -> Self;
}

//an object to be put into a schedule and called later
pub type SchedFn<SrcSnk, Context> = Box<SchedCall<SrcSnk, Context>>;

//an object that can schedule SchedFn's and provide a SrcSnk with the src_sink() method
pub trait Sched<SrcSnk, Context> {
    fn schedule(&mut self, t: TimeSched, func: SchedFn<SrcSnk, Context>);
}

pub trait ExecSched<SrcSnk, Context>: Sched<SrcSnk, Context> {
    fn src_sink(&mut self) -> &mut SrcSnk;
    fn context(&mut self) -> Context;
}

pub trait SchedCall<SrcSnk, Context>: Send {
    fn sched_call(
        &mut self,
        sched: &mut ExecSched<SrcSnk, Context>,
        context: &mut Context,
    ) -> TimeResched;
}

pub trait NodeSrcSnk<SrcSnk, Context> {
    fn pop_node(&mut self) -> Option<SchedFnNode<SrcSnk, Context>>;
}

pub trait SrcSnkUpdate: Send {
    fn update(&mut self) -> bool;
}

pub trait SrcSnkCreate<SrcSnk, Update: SrcSnkUpdate> {
    fn src_sink(&mut self) -> Option<SrcSnk>;
    fn updater(&mut self) -> Option<Update>;
}

//implement sched_call for any Fn that with the correct sig
impl<
    F: Fn(&mut ExecSched<SrcSnk, Context>, &mut Context) -> TimeResched,
    SrcSnk,
    Context: ContextInit,
> SchedCall<SrcSnk, Context> for F
where
    F: Send,
{
    fn sched_call(
        &mut self,
        sched: &mut ExecSched<SrcSnk, Context>,
        context: &mut Context,
    ) -> TimeResched {
        (*self)(sched, context)
    }
}

pub struct TimedFn<SrcSnk, Context> {
    time: usize,
    func: Option<SchedFn<SrcSnk, Context>>,
}
pub type SchedFnNode<SrcSnk, Context> = Box<xnor_llist::Node<TimedFn<SrcSnk, Context>>>;

impl<SrcSnk, Context> Default for TimedFn<SrcSnk, Context> {
    fn default() -> Self {
        TimedFn {
            time: 0,
            func: None,
        }
    }
}

pub struct Executor<SrcSnk, Context>
where
    SrcSnk: NodeSrcSnk<SrcSnk, Context>,
    Context: ContextInit,
{
    list: List<TimedFn<SrcSnk, Context>>,
    time: Arc<AtomicUsize>,
    receiver: Receiver<SchedFnNode<SrcSnk, Context>>,
    src_sink: SrcSnk,
    dispose_sender: SyncSender<Box<Send>>,
    phantom_context: std::marker::PhantomData<Context>,
}

pub struct Scheduler<SrcSnkCreator, SrcSnk, Context, Update>
where
    SrcSnkCreator: SrcSnkCreate<SrcSnk, Update> + Default,
    SrcSnk: NodeSrcSnk<SrcSnk, Context>,
    Update: SrcSnkUpdate + 'static,
    Context: ContextInit,
{
    time: Arc<AtomicUsize>,
    src_sink: SrcSnkCreator,
    executor: Option<Executor<SrcSnk, Context>>,
    sender: SyncSender<SchedFnNode<SrcSnk, Context>>,
    dispose_receiver: Option<Receiver<Box<Send>>>,
    dispose_handle: Option<thread::JoinHandle<()>>,
    src_sink_handle: Option<thread::JoinHandle<()>>,
    phantom_update: std::marker::PhantomData<Update>,
}

impl<SrcSnkCreator, SrcSnk, Context, Update> Scheduler<SrcSnkCreator, SrcSnk, Context, Update>
where
    SrcSnkCreator: SrcSnkCreate<SrcSnk, Update> + Default,
    SrcSnk: NodeSrcSnk<SrcSnk, Context>,
    Update: SrcSnkUpdate + 'static,
    Context: ContextInit,
{
    pub fn new() -> Self {
        let (sender, receiver) = sync_channel(1024);
        let (dispose_sender, dispose_receiver) = sync_channel(1024);
        let mut src_sink = SrcSnkCreator::default();
        let time = Arc::new(AtomicUsize::new(0));
        Scheduler {
            time: time.clone(),
            executor: Some(Executor {
                list: List::new(),
                time: time,
                receiver,
                src_sink: src_sink.src_sink().unwrap(),
                dispose_sender,
                phantom_context: std::marker::PhantomData,
            }),
            sender,
            src_sink: src_sink,
            dispose_receiver: Some(dispose_receiver),
            dispose_handle: None,
            src_sink_handle: None,
            phantom_update: std::marker::PhantomData,
        }
    }

    pub fn executor(&mut self) -> Option<Executor<SrcSnk, Context>> {
        self.executor.take()
    }

    /// Spawn the helper threads
    pub fn spawn_helper_threads(&mut self) -> () {
        self.spawn_dispose_thread();
        self.spawn_src_sink_thread();
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

    /// Spawn a thread to fill up the src_sink so we can get objects in the execution thread
    /// Note: This calls update once in the current thread in order to get the src_sink full
    /// immediately
    pub fn spawn_src_sink_thread(&mut self) -> () {
        if self.src_sink_handle.is_some() {
            return;
        }

        let mut updater = self.src_sink.updater().unwrap();
        updater.update(); //get an initial update
        self.src_sink_handle = Some(thread::spawn(move || {
            let sleep_time = std::time::Duration::from_millis(5);
            while updater.update() {
                thread::sleep(sleep_time);
            }
        }));
    }
}

impl<SrcSnk: 'static, Context: 'static> Executor<SrcSnk, Context>
where
    SrcSnk: NodeSrcSnk<SrcSnk, Context> + 'static,
    Context: ContextInit,
{
    fn add_node(&mut self, node: SchedFnNode<SrcSnk, Context>) {
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
            let mut context = Context::with_time(0);
            match timedfn.sched_call(self, &mut context) {
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

impl<SrcSnk, Context> SchedCall<SrcSnk, Context> for TimedFn<SrcSnk, Context> {
    fn sched_call(
        &mut self,
        s: &mut ExecSched<SrcSnk, Context>,
        context: &mut Context,
    ) -> TimeResched {
        if let Some(ref mut f) = self.func {
            f.sched_call(s, context)
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

impl<SrcSnkCreator, SrcSnk, Context, Update> Sched<SrcSnk, Context>
    for Scheduler<SrcSnkCreator, SrcSnk, Context, Update>
where
    SrcSnkCreator: SrcSnkCreate<SrcSnk, Update> + Default,
    SrcSnk: NodeSrcSnk<SrcSnk, Context>,
    Update: SrcSnkUpdate + 'static,
    Context: ContextInit,
{
    fn schedule(&mut self, time: TimeSched, func: SchedFn<SrcSnk, Context>) {
        let f = Node::new_boxed(TimedFn {
            func: Some(func),
            time: add_time(&self.time, &time),
        });
        self.sender.send(f).unwrap();
    }
}

impl<SrcSnk, Context> Sched<SrcSnk, Context> for Executor<SrcSnk, Context>
where
    SrcSnk: NodeSrcSnk<SrcSnk, Context>,
    Context: ContextInit,
{
    fn schedule(&mut self, time: TimeSched, func: SchedFn<SrcSnk, Context>) {
        match self.src_sink.pop_node() {
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

impl<SrcSnk, Context> ExecSched<SrcSnk, Context> for Executor<SrcSnk, Context>
where
    SrcSnk: NodeSrcSnk<SrcSnk, Context>,
    Context: ContextInit,
{
    fn src_sink(&mut self) -> &mut SrcSnk {
        &mut self.src_sink
    }

    fn context(&mut self) -> Context {
        Context::with_time(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn can_vec() {
        let _x: Vec<TimedFn<(), ()>> = (0..20).map({ |_| TimedFn::default() }).collect();
    }

    impl NodeSrcSnk<(), ()> for () {
        fn pop_node(&mut self) -> Option<SchedFnNode<(), ()>> {
            Some(Node::new_boxed(Default::default()))
        }
    }

    impl SrcSnkUpdate for () {
        fn update(&mut self) -> bool {
            true
        }
    }

    impl ContextInit for () {
        fn with_time(_time: usize) -> () {
            ()
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
            e.run(32);
            e.run(32);
        });

        assert!(child.join().is_ok());
    }
}
