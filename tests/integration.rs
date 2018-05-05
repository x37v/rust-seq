#![feature(specialization)]
#![feature(nll)]

extern crate xnor_seq;

use xnor_seq::{CacheCreate, CacheUpdate, ExecSched, Node, NodeCache, Sched, SchedFnNode,
               Scheduler, TimeResched, TimeSched};
use std::sync::mpsc::{sync_channel, Receiver, SyncSender, TrySendError};
use std::thread;

type TestSink = ();
type TestContext = ();

struct TestCache {
    receiver: Receiver<SchedFnNode<TestCache, TestSink, TestContext>>,
}

struct TestCacheUpdater {
    sender: SyncSender<SchedFnNode<TestCache, TestSink, TestContext>>,
}

struct TestCacheCreator {
    sender: SyncSender<SchedFnNode<TestCache, TestSink, TestContext>>,
    cache: Option<TestCache>,
}

impl NodeCache<TestCache, TestSink, TestContext> for TestCache {
    fn pop_node(&mut self) -> Option<SchedFnNode<TestCache, TestSink, TestContext>> {
        self.receiver.try_recv().ok()
    }
}

impl CacheUpdate for TestCacheUpdater {
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

impl CacheCreate<TestCache, TestCacheUpdater> for TestCacheCreator {
    fn cache(&mut self) -> Option<TestCache> {
        self.cache.take()
    }

    fn updater(&mut self) -> Option<TestCacheUpdater> {
        Some(TestCacheUpdater {
            sender: self.sender.clone(),
        })
    }
}

impl Default for TestCacheCreator {
    fn default() -> Self {
        let (sender, receiver) = sync_channel(1024);
        TestCacheCreator {
            sender: sender,
            cache: Some(TestCache { receiver: receiver }),
        }
    }
}

#[test]
fn real_cache() {
    type SImpl = Scheduler<TestCacheCreator, TestCache, TestSink, TestContext, TestCacheUpdater>;
    type EImpl<'a> = ExecSched<TestCache, TestSink, TestContext> + 'a;

    let mut s = SImpl::new();
    s.spawn_helper_threads();

    let e = s.executor();
    assert!(e.is_some());
    s.schedule(
        TimeSched::Absolute(0),
        Box::new(move |s: &mut EImpl| {
            println!("Closure in schedule");
            assert!(s.cache().pop_node().is_some());
            assert_eq!(s.sink(), &());
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
