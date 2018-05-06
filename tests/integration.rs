#![feature(specialization)]
#![feature(nll)]

extern crate xnor_seq;

use xnor_seq::{ContextInit, ExecSched, Node, NodeSrcSnk, Sched, SchedFnNode, Scheduler,
               SrcSnkCreate, SrcSnkUpdate, TimeResched, TimeSched};
use std::sync::mpsc::{sync_channel, Receiver, SyncSender, TrySendError};
use std::thread;

#[derive(Debug)]
struct TestContext;

struct TestSrcSnk {
    receiver: Receiver<SchedFnNode<TestSrcSnk, TestContext>>,
}

struct TestSrcSnkUpdater {
    sender: SyncSender<SchedFnNode<TestSrcSnk, TestContext>>,
}

struct TestSrcSnkCreator {
    sender: SyncSender<SchedFnNode<TestSrcSnk, TestContext>>,
    src_sink: Option<TestSrcSnk>,
}

impl NodeSrcSnk<TestSrcSnk, TestContext> for TestSrcSnk {
    fn pop_node(&mut self) -> Option<SchedFnNode<TestSrcSnk, TestContext>> {
        self.receiver.try_recv().ok()
    }
}

impl ContextInit for TestContext {
    fn with_time(_time: usize) -> TestContext {
        TestContext
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
            let f = Node::new_boxed(Default::default());
            match self.sender.try_send(f) {
                Ok(_) => {}
                Err(TrySendError::Full(_)) => return true,
                Err(TrySendError::Disconnected(_)) => return false,
            }
        }
    }
}

impl SrcSnkCreate<TestSrcSnk, TestSrcSnkUpdater> for TestSrcSnkCreator {
    fn src_sink(&mut self) -> Option<TestSrcSnk> {
        self.src_sink.take()
    }

    fn updater(&mut self) -> Option<TestSrcSnkUpdater> {
        Some(TestSrcSnkUpdater {
            sender: self.sender.clone(),
        })
    }
}

impl Default for TestSrcSnkCreator {
    fn default() -> Self {
        let (sender, receiver) = sync_channel(1024);
        TestSrcSnkCreator {
            sender: sender,
            src_sink: Some(TestSrcSnk { receiver: receiver }),
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
        e.run(32);
        e.run(32);
    });

    assert!(child.join().is_ok());
}
