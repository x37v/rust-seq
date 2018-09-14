#![feature(specialization)]
#![feature(nll)]

extern crate sched;

use sched::{
    ContextBase, DisposeSink, ExecSched, Node, NodeSrc, Sched, SchedFnNode, Scheduler,
    SrcSnkCreate, SrcSnkUpdate, TimeResched, TimeSched,
};
use std::sync::mpsc::{sync_channel, Receiver, SyncSender, TrySendError};
use std::thread;

#[derive(Debug)]
struct TestContext;

struct TestSrcSnk {
    receiver: Receiver<SchedFnNode<TestSrcSnk, TestContext>>,
    dispose_send: SyncSender<Box<Send>>,
}

struct TestSrcSnkUpdater {
    sender: SyncSender<SchedFnNode<TestSrcSnk, TestContext>>,
    dispose_recv: Receiver<Box<Send>>,
}

struct TestSrcSnkCreator {
    updater: Option<TestSrcSnkUpdater>,
    src_sink: Option<TestSrcSnk>,
}

impl NodeSrc<TestSrcSnk, TestContext> for TestSrcSnk {
    fn pop_node(&mut self) -> Option<SchedFnNode<TestSrcSnk, TestContext>> {
        self.receiver.try_recv().ok()
    }
}

impl DisposeSink for TestSrcSnk {
    fn dispose(&mut self, item: Box<Send>) {
        self.dispose_send.try_send(item).ok();
    }
}

impl ContextBase for TestContext {
    fn from_root(_tick: usize, _ticks_per_second: usize) -> Self {
        TestContext
    }
    fn from_parent<T: ContextBase>(_parent: &T) -> Self {
        TestContext
    }
    fn with_tick<T: ContextBase>(_tick: usize, _parent: &T) -> Self {
        TestContext
    }
    fn tick(&self) -> usize {
        0
    }
    fn ticks_per_second(&self) -> Option<usize> {
        Some(44100)
    }
}

impl TestContext {
    fn doit(&self) {
        println!("TestContext");
    }
}

impl SrcSnkUpdate for TestSrcSnkUpdater {
    fn update(&mut self) -> bool {
        loop {
            let mut ret = None;
            let f = Node::new_boxed(Default::default());
            match self.sender.try_send(f) {
                Ok(_) => {}
                Err(TrySendError::Full(_)) => ret = Some(true),
                Err(TrySendError::Disconnected(_)) => ret = Some(false),
            }
            //ditch boxed items, letting them be dropped
            if let Ok(_) = self.dispose_recv.try_recv() {
                println!("got dispose");
            }
            if let Some(v) = ret {
                return v;
            }
        }
    }
}

impl SrcSnkCreate<TestSrcSnk, TestSrcSnkUpdater> for TestSrcSnkCreator {
    fn src_sink(&mut self) -> Option<TestSrcSnk> {
        self.src_sink.take()
    }

    fn updater(&mut self) -> Option<TestSrcSnkUpdater> {
        self.updater.take()
    }
}

impl Default for TestSrcSnkCreator {
    fn default() -> Self {
        let (sender, receiver) = sync_channel(1024);
        let (dispose_send, dispose_recv) = sync_channel(1024);
        TestSrcSnkCreator {
            updater: Some(TestSrcSnkUpdater {
                sender,
                dispose_recv,
            }),
            src_sink: Some(TestSrcSnk {
                receiver: receiver,
                dispose_send,
            }),
        }
    }
}

#[test]
fn real_src_sink() {
    type SImpl = Scheduler<TestSrcSnkCreator, TestSrcSnk, TestContext, TestSrcSnkUpdater>;
    type EImpl<'a> = ExecSched<TestSrcSnk, TestContext> + 'a;

    let mut s = SImpl::new();
    s.spawn_helper_threads();

    let e = s.executor();
    assert!(e.is_some());
    s.schedule(
        TimeSched::Absolute(0),
        Box::new(move |s: &mut EImpl, context: &mut TestContext| {
            println!("Closure in schedule");
            assert!(s.src_sink().pop_node().is_some());
            context.doit();
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
