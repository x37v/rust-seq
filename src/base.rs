use crate::binding::set::BindingSet;
use crate::context::SchedContext;
use crate::ptr::*;
use crate::time::{TimeResched, TimeSched};
use crate::trigger::{Trigger, TriggerId};

pub enum Value {
    Byte(u8),
    Int32(i32),
    Int64(i64),
    UInt32(u32),
    UInt64(u64),
    Float32(f32),
    Float64(f64),
    Char(char),
    Bool(bool),
}

pub trait Sched {
    fn schedule(&mut self, t: TimeSched, func: SchedFn);
}

pub trait SchedCall: Send {
    fn sched_call(&mut self, context: &mut dyn SchedContext) -> TimeResched;
}

//an object to be put into a schedule and called later
pub type SchedFn = UniqPtr<dyn SchedCall>;

cfg_if! {
    if #[cfg(feature = "with_std")] {

        pub trait TimedNodeData {
            fn set_time(&mut self, time: usize);
            fn time(&self) -> usize;
        }

        pub trait InsertTimeSorted<T> {
            fn insert_time_sorted(&mut self, node: UniqPtr<LNode<T>>);
        }

        pub type TimedTrigNode = UniqPtr<LNode<TimedTrig>>;
        pub type SchedFnNode = UniqPtr<LNode<TimedFn>>;
        pub type BindingSetNode = UniqPtr<LNode<BindingSet>>;
        pub type TriggerNode = UniqPtr<LNode<SShrPtr<dyn Trigger>>>;

        use xnor_llist;

        pub use xnor_llist::List as LList;
        pub use xnor_llist::Node as LNode;

        use std;
        use std::sync::atomic::AtomicUsize;
        use std::sync::mpsc::{sync_channel, Receiver, SyncSender, TryRecvError, TrySendError};
        use std::thread;
        use crate::executor::Executor;


        //implement sched_call for any Fn that with the correct sig
        impl<F: Fn(&mut dyn SchedContext) -> TimeResched> SchedCall for F
            where
                F: Send,
            {
                fn sched_call(&mut self, context: &mut dyn SchedContext) -> TimeResched {
                    (*self)(context)
                }
            }

        pub struct TimedFn {
            pub time: usize,
            pub func: Option<SchedFn>,
        }

        impl<T> InsertTimeSorted<T> for LList<T>
            where
                T: TimedNodeData,
            {
                fn insert_time_sorted(&mut self, node: UniqPtr<LNode<T>>) {
                    self.insert(node, |n, o| n.time() <= o.time());
                }
            }

        impl TimedFn {
            pub fn set_func(&mut self, func: Option<SchedFn>) {
                self.func = func
            }
            pub fn func(&mut self) -> Option<SchedFn> {
                self.func.take()
            }
        }

        impl TimedNodeData for TimedFn {
            fn set_time(&mut self, time: usize) {
                self.time = time;
            }
            fn time(&self) -> usize {
                self.time
            }
        }

        impl Default for TimedFn {
            fn default() -> Self {
                Self {
                    time: 0,
                    func: None,
                }
            }
        }

        pub struct TimedTrig {
            time: usize,
            index: Option<TriggerId>,
            values: LList<BindingSet>,
        }

        impl TimedTrig {
            pub fn set_index(&mut self, index: Option<TriggerId>) {
                self.index = index;
            }
            pub fn index(&self) -> Option<TriggerId> {
                self.index
            }
            pub fn add_value(&mut self, vnode: BindingSetNode) {
                self.values.push_front(vnode);
            }
            pub fn values(&self) -> &LList<BindingSet> {
                &self.values
            }
        }

        impl TimedNodeData for TimedTrig {
            fn set_time(&mut self, time: usize) {
                self.time = time;
            }
            fn time(&self) -> usize {
                self.time
            }
        }

        impl Default for TimedTrig {
            fn default() -> Self {
                Self {
                    time: 0,
                    index: None,
                    values: LList::new(),
                }
            }
        }

        pub struct SrcSink {
            node_cache: Receiver<SchedFnNode>,
            trig_cache: Receiver<TimedTrigNode>,
            value_set_cache: Receiver<BindingSetNode>,
            dispose_schedule_sender: SyncSender<UniqPtr<dyn Send>>,
            updater: Option<SrcSinkUpdater>,
        }

        pub struct SrcSinkUpdater {
            node_cache_updater: SyncSender<SchedFnNode>,
            trig_cache_updater: SyncSender<TimedTrigNode>,
            value_set_cache_updater: SyncSender<BindingSetNode>,
            dispose_schedule_receiver: Receiver<UniqPtr<dyn Send>>,
        }

        pub struct Scheduler {
            time: ShrPtr<AtomicUsize>,
            executor: Option<Executor>,
            schedule_sender: SyncSender<SchedFnNode>,
            updater: Option<SrcSinkUpdater>,
            helper_handle: Option<thread::JoinHandle<()>>,
        }

        impl SrcSinkUpdater {
            pub fn new(
                node_cache_updater: SyncSender<SchedFnNode>,
                trig_cache_updater: SyncSender<TimedTrigNode>,
                value_set_cache_updater: SyncSender<BindingSetNode>,
                dispose_schedule_receiver: Receiver<UniqPtr<dyn Send>>,
                ) -> Self {
                Self {
                    node_cache_updater,
                    trig_cache_updater,
                    value_set_cache_updater,
                    dispose_schedule_receiver,
                }
            }

            pub fn update(&self) -> bool {
                loop {
                    match self.dispose_schedule_receiver.try_recv() {
                        Ok(_) => continue,
                        Err(TryRecvError::Empty) => (),
                        Err(TryRecvError::Disconnected) => return false,
                    }
                    match self.node_cache_updater.try_send(Default::default()) {
                        Ok(_) => continue,
                        Err(TrySendError::Full(_)) => (),
                        Err(TrySendError::Disconnected(_)) => return false,
                    }
                    match self.trig_cache_updater.try_send(Default::default()) {
                        Ok(_) => continue,
                        Err(TrySendError::Full(_)) => (),
                        Err(TrySendError::Disconnected(_)) => return false,
                    }
                    match self.value_set_cache_updater.try_send(Default::default()) {
                        Ok(_) => continue,
                        Err(TrySendError::Full(_)) => (),
                        Err(TrySendError::Disconnected(_)) => return false,
                    }
                    break;
                }
                true
            }
        }

        impl SrcSink {
            pub fn new() -> Self {
                let (dispose_schedule_sender, dispose_schedule_receiver) = sync_channel(1024);
                let (node_cache_updater, node_cache) = sync_channel(1024);
                let (trig_cache_updater, trig_cache) = sync_channel(1024);
                let (value_set_cache_updater, value_set_cache) = sync_channel(1024);
                Self {
                    node_cache,
                    trig_cache,
                    value_set_cache,
                    dispose_schedule_sender,
                    updater: Some(SrcSinkUpdater::new(
                            node_cache_updater,
                            trig_cache_updater,
                            value_set_cache_updater,
                            dispose_schedule_receiver,
                            )),
                }
            }

            pub fn updater(&mut self) -> Option<SrcSinkUpdater> {
                self.updater.take()
            }

            pub fn pop_node(&mut self) -> Option<SchedFnNode> {
                self.node_cache.try_recv().ok()
            }

            pub fn pop_trig(&mut self) -> Option<TimedTrigNode> {
                self.trig_cache.try_recv().ok()
            }

            pub fn pop_value_set(&mut self) -> Option<BindingSetNode> {
                self.value_set_cache.try_recv().ok()
            }

            pub fn dispose(&mut self, item: UniqPtr<dyn Send>) {
                let _ = self.dispose_schedule_sender.send(item);
            }
        }

        impl Default for SrcSink {
            fn default() -> SrcSink {
                SrcSink::new()
            }
        }

        impl Scheduler {
            pub fn new() -> Self {
                let (schedule_sender, schedule_receiver) = sync_channel(1024);
                let time = ShrPtr::new(AtomicUsize::new(0));
                let mut src_sink = SrcSink::new();
                let updater = src_sink.updater();
                Scheduler {
                    time: time.clone(),
                    executor: Some(Executor::new(time, schedule_receiver, src_sink)),
                    schedule_sender,
                    updater,
                    helper_handle: None,
                }
            }

            pub fn spawn_helper_threads(&mut self) {
                if let Some(updater) = self.updater.take() {
                    //fill the caches, then spawn a thread to keep it updated
                    updater.update();
                    self.helper_handle = Some(thread::spawn(move || {
                        let sleep_time = std::time::Duration::from_millis(5);
                        while updater.update() {
                            thread::sleep(sleep_time);
                        }
                    }));
                }
            }

            pub fn executor(&mut self) -> Option<Executor> {
                self.executor.take()
            }
        }

        impl Default for Scheduler {
            fn default() -> Self {
                Self::new()
            }
        }

        impl SchedCall for TimedFn {
            fn sched_call(&mut self, context: &mut dyn SchedContext) -> TimeResched {
                if let Some(ref mut f) = self.func {
                    f.sched_call(context)
                } else {
                    TimeResched::None
                }
            }
        }

        impl Sched for Scheduler {
            fn schedule(&mut self, time: TimeSched, func: SchedFn) {
                let f = LNode::new_boxed(TimedFn {
                    func: Some(func),
                    time: crate::util::add_atomic_time(&self.time, &time),
                });
                self.schedule_sender.send(f).unwrap();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn can_vec() {
        let _x: Vec<TimedFn> = (0..20).map({ |_| TimedFn::default() }).collect();
    }

    /*
    #[test]
    fn basic_test() {
        let mut s = Scheduler::new();
        s.spawn_helper_threads();

        let e = s.executor();
        let trig = TriggerId::new();
        assert!(e.is_some());
        s.schedule(
            TimeSched::Absolute(0),
            UniqPtr::new(move |context: &mut dyn SchedContext| {
                println!(
                    "Closure in schedule {}, {}",
                    context.base_tick(),
                    context.base_tick_period_micros()
                );
                let at = context.base_tick();
                context.schedule(
                    TimeSched::Relative(12),
                    //XXX shouldn't actually allocate this
                    UniqPtr::new(move |context: &mut dyn SchedContext| {
                        println!("inner dog {}, scheduled at {}", context.base_tick(), at);
                        context.schedule_trigger(TimeSched::Relative(0), trig);
                        TimeResched::None
                    }),
                );
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
    */
}
