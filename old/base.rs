use crate::context::SchedContext;
use crate::ptr::*;
use crate::time::{TimeResched, TimeSched};
use crate::trigger::{TrigCall, Trigger};

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
pub type TrigCallPtr = UniqPtr<dyn TrigCall>;
pub type TrigPtr = SShrPtr<dyn Trigger>;

cfg_if! {
    if #[cfg(feature = "std")] {

        use std;
        use std::sync::atomic::AtomicUsize;
        use std::sync::mpsc::{sync_channel, Receiver, SyncSender, TryRecvError, TrySendError};
        use std::thread;
        use crate::executor::Executor;
        use crate::llist_pqueue::LListPQueue;

        //XXX TODO, make this UniqPtr<Trig> without any time
        type LListExecutor = Executor<LListPQueue<SchedFn>, LListPQueue<TrigCallPtr>>;

        //implement sched_call for any Fn that with the correct sig
        impl<F: Fn(&mut dyn SchedContext) -> TimeResched> SchedCall for F
            where
                F: Send,
            {
                fn sched_call(&mut self, context: &mut dyn SchedContext) -> TimeResched {
                    (*self)(context)
                }
            }

        pub struct SrcSink {
            dispose_schedule_sender: SyncSender<UniqPtr<dyn Send>>,
            updater: Option<SrcSinkUpdater>,
        }

        pub struct SrcSinkUpdater {
            dispose_schedule_receiver: Receiver<UniqPtr<dyn Send>>,
        }

        pub struct Scheduler {
            time: ShrPtr<AtomicUsize>,
            executor: Option<LListExecutor>,
            schedule_sender: SyncSender<(usize,SchedFn)>,
            updater: Option<SrcSinkUpdater>,
            helper_handle: Option<thread::JoinHandle<()>>,
        }

        impl SrcSinkUpdater {
            pub fn new(
                dispose_schedule_receiver: Receiver<UniqPtr<dyn Send>>,
                ) -> Self {
                Self {
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
                    /*
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
                    */
                    break;
                }
                true
            }
        }

        impl SrcSink {
            pub fn new() -> Self {
                let (dispose_schedule_sender, dispose_schedule_receiver) = sync_channel(1024);
                Self {
                    dispose_schedule_sender,
                    updater: Some(SrcSinkUpdater::new(
                            dispose_schedule_receiver,
                            )),
                }
            }

            pub fn updater(&mut self) -> Option<SrcSinkUpdater> {
                self.updater.take()
            }

            /*
            pub fn pop_node(&mut self) -> Option<SchedFnNode> {
                self.node_cache.try_recv().ok()
            }

            pub fn pop_trig(&mut self) -> Option<TimedTrigNode> {
                self.trig_cache.try_recv().ok()
            }

            pub fn pop_value_set(&mut self) -> Option<BindingSetNode> {
                self.value_set_cache.try_recv().ok()
            }
            */

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
                let eschedule = LListPQueue::new();
                let etrig_schedule = LListPQueue::new();
                Scheduler {
                    time: time.clone(),
                    executor: Some(Executor::new(
                            eschedule, etrig_schedule,
                            time, schedule_receiver, src_sink)),
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

            pub fn executor(&mut self) -> Option<LListExecutor> {
                self.executor.take()
            }
        }

        impl Default for Scheduler {
            fn default() -> Self {
                Self::new()
            }
        }

        impl Sched for Scheduler {
            fn schedule(&mut self, time: TimeSched, func: SchedFn) {
                /*
                 * TODO
                let f = LNode::new_boxed(TimedFn {
                    func: Some(func),
                    time: crate::util::add_atomic_time(&self.time, &time),
                });
                self.schedule_sender.send(f).unwrap();
                */
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    /*
    #[test]
    fn can_vec() {
        let _x: Vec<TimedFn> = (0..20).map({ |_| TimedFn::default() }).collect();
    }

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
