#![feature(specialization)]
#![feature(nll)]

extern crate sched;
extern crate sched_tree;

use sched::{
    ContextBase, DisposeSink, ExecSched, Node, NodeSrc, Sched, SchedFnNode, Scheduler,
    SrcSnkCreate, SrcSnkUpdate, TimeResched, TimeSched,
};
use sched_tree::Clock;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{sync_channel, Receiver, SyncSender, TrySendError};
use std::sync::Arc;
use std::thread;

#[derive(Debug)]
struct TestContext {
    time: usize,
    ticks_per_second: usize,
}

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
    fn with_time(time: usize, ticks_per_second: usize) -> TestContext {
        TestContext {
            time,
            ticks_per_second,
        }
    }
    fn ticks_per_second(&self) -> usize {
        self.ticks_per_second
    }
}

impl TestContext {
    fn doit(&self) {
        println!("TestContext");
    }

    fn now(&self) -> usize {
        self.time
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
    let period = Arc::new(AtomicUsize::new(5_000));
    let pc = period.clone();

    let (mut ctl, clk) = Clock::new(10_000,
            Box::new(move |s: &mut EImpl, context: &mut TestContext| {
                println!("controlled {}", context.now());
                if context.now() < 1_000_000 {
                    TimeResched::Relative(0)
                } else {
                    TimeResched::None
                }
            })
    );

    s.schedule(TimeSched::Absolute(10_000), Box::new(clk));

    ctl.set_period(1_000_000);

    s.schedule(
        TimeSched::Absolute(0),
        Box::new(Clock::new_micros(
            period,
            Box::new(move |s: &mut EImpl, context: &mut TestContext| {
                let p = pc.load(Ordering::SeqCst);
                println!(
                    "Clocked Closure in schedule: {}, period {}",
                    context.now(),
                    p
                );
                pc.store(p + 1_000, Ordering::SeqCst);
                if context.now() < 441000 {
                    TimeResched::Relative(0)
                } else {
                    println!("UNSCHED");
                    TimeResched::None
                }
            }),
        )),
    );

    let child = thread::spawn(move || {
        let mut e = e.unwrap();
        for _ in (1..100) {
            e.run(44100, 44100);
        }
    });

    assert!(child.join().is_ok());
}
