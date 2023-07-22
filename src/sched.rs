//! Schedules
use crate::context::RootContext;
use crate::event::*;
use crate::pqueue::{TickPriorityDequeue, TickPriorityEnqueue};
use crate::tick::*;

/// Schedule executor.
pub struct SchedExec<R, W, E, U>
where
    R: TickPriorityDequeue<E>,
    W: TickPriorityEnqueue<E>,
{
    tick_last: usize,
    schedule_reader: R,
    schedule_writer: W,
    _phantom: core::marker::PhantomData<fn() -> (E, U)>,
}

impl<R, W, E, U> SchedExec<R, W, E, U>
where
    R: TickPriorityDequeue<E>,
    W: TickPriorityEnqueue<E>,
    E: EventEval<E, U>,
{
    pub fn new(schedule_reader: R, schedule_writer: W, start_tick: usize) -> Self {
        Self {
            tick_last: start_tick,
            schedule_reader,
            schedule_writer,
            _phantom: Default::default(),
        }
    }

    /// Run until `tick_offset` (relative) ticks from the last run
    pub fn run_offset(&mut self, tick_offset: usize, ticks_per_second: usize, user_data: &mut U) {
        self.run(
            self.tick_last.wrapping_add(tick_offset),
            ticks_per_second,
            user_data,
        )
    }

    /// Run until `tick` (absolute)
    pub fn run(&mut self, tick: usize, ticks_per_second: usize, user_data: &mut U) {
        //eval from the last tick until the given tick
        //TODO there are likely other places where we have to deal with rollover
        let t0 = self.tick_last;
        let t1 = if tick < t0 { core::usize::MAX } else { tick };

        let mut context = RootContext::new(t0, ticks_per_second, &mut self.schedule_writer);

        //evaluate events before next
        while let Some((t, mut event)) = self.schedule_reader.dequeue_lt(t1) {
            //clamp below t0, exal and dispose
            let tick = t.max(t0);
            context.update_tick(tick);

            //eval and see about rescheduling
            let r = match event.event_eval(&mut context, user_data) {
                TickResched::Relative(t) => Some(TickSched::Relative(t as isize)),
                TickResched::ContextRelative(t) => Some(TickSched::ContextRelative(t as isize)),
                TickResched::None => None,
            };

            //try to reschedule if we should
            if let Some(t) = r {
                let _ = context.event_try_schedule(t, event);
            }
        }

        self.tick_last = t1;
    }

    pub fn tick_last(&self) -> usize {
        self.tick_last
    }
}

#[cfg(all(test, feature = "with_alloc", feature = "std"))]
mod tests {
    use super::*;
    use crate::{
        event::{boxed::EventContainer, EventEval, EventEvalContext},
        graph::root::{clock::RootClock, GraphRootWrapper},
        pqueue::binaryheap::BinaryHeapQueue,
    };
    use core::cmp::Ordering;
    use std::sync::{
        atomic::{AtomicUsize, Ordering as AOrdering},
        Mutex,
    };

    static ENUM_CNT: AtomicUsize = AtomicUsize::new(0);
    static REF_CNT: AtomicUsize = AtomicUsize::new(0);

    lazy_static::lazy_static! {
        static ref CLOCK_REF: Mutex<GraphRootWrapper<RootClock<f64, bool, bool>, (), RefEventContainer, ()>> = {
            let c = Mutex::new(GraphRootWrapper::new(RootClock::new(1000f64, true, false), ()));
            c
        };
        static ref CLOCK_ENUM: Mutex<GraphRootWrapper<RootClock<f64, bool, bool>, (), EnumEvent, ()>> = {
            let c = Mutex::new(GraphRootWrapper::new(RootClock::new(1000f64, true, false), ()));
            c
        };
    }

    enum EnumEvent {
        Root(&'static Mutex<GraphRootWrapper<RootClock<f64, bool, bool>, (), EnumEvent, ()>>),
    }

    pub struct RefEventContainer {
        inner: &'static Mutex<dyn EventEval<RefEventContainer, ()> + Send>,
    }

    pub fn ptr_cmp<T: ?Sized>(a: *const T, b: *const T) -> Ordering {
        a.cmp(&b)
    }

    impl Ord for EnumEvent {
        fn cmp(&self, other: &Self) -> Ordering {
            match other {
                Self::Root(o) => match self {
                    Self::Root(r) => ptr_cmp(r, o),
                },
            }
        }
    }

    impl PartialOrd for EnumEvent {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }

    impl PartialEq for EnumEvent {
        fn eq(&self, _other: &Self) -> bool {
            false //box, never equal
        }
    }

    impl Eq for EnumEvent {}

    impl EventEval<EnumEvent, ()> for EnumEvent {
        fn event_eval(
            &mut self,
            context: &mut dyn EventEvalContext<EnumEvent>,
            user_data: &mut (),
        ) -> TickResched {
            ENUM_CNT.fetch_add(1, AOrdering::SeqCst);
            match &self {
                Self::Root(r) => r.lock().unwrap().event_eval(context, user_data),
            }
        }
    }

    impl EventEval<RefEventContainer, ()> for RefEventContainer {
        fn event_eval(
            &mut self,
            context: &mut dyn EventEvalContext<Self>,
            user_data: &mut (),
        ) -> TickResched {
            REF_CNT.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            self.inner.lock().unwrap().event_eval(context, user_data)
        }
    }

    impl RefEventContainer {
        pub fn new(inner: &'static Mutex<dyn EventEval<RefEventContainer, ()> + Send>) -> Self {
            Self { inner }
        }
    }

    impl Ord for RefEventContainer {
        fn cmp(&self, other: &Self) -> Ordering {
            ptr_cmp(self.inner, other.inner)
        }
    }

    impl PartialOrd for RefEventContainer {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }

    impl PartialEq for RefEventContainer {
        fn eq(&self, _other: &Self) -> bool {
            false //box, never equal
        }
    }

    impl Eq for RefEventContainer {}

    #[test]
    fn can_build_boxed() {
        let clock = GraphRootWrapper::new(RootClock::new(1.0 as crate::Float, true, false), ());
        let mut reader: BinaryHeapQueue<EventContainer<()>> = BinaryHeapQueue::with_capacity(16);
        let writer: BinaryHeapQueue<EventContainer<()>> = BinaryHeapQueue::default();

        assert!(reader
            .try_enqueue(0, EventContainer::new(Box::new(clock)))
            .is_ok());
        let mut sched = SchedExec::new(reader, writer, 0);
        sched.run(0, 16, &mut ());
    }

    #[test]
    fn can_build_enum() {
        let mut reader: BinaryHeapQueue<EnumEvent> = BinaryHeapQueue::with_capacity(16);
        let writer: BinaryHeapQueue<EnumEvent> = BinaryHeapQueue::default();

        ENUM_CNT.store(0, AOrdering::SeqCst);

        assert!(reader.try_enqueue(0, EnumEvent::Root(&*CLOCK_ENUM)).is_ok());
        let mut sched = SchedExec::new(reader, writer, 0);

        sched.run(0, 44100, &mut ());
        assert_eq!(ENUM_CNT.load(AOrdering::SeqCst), 0);
        sched.run(16, 44100, &mut ());
        assert_eq!(ENUM_CNT.load(AOrdering::SeqCst), 1);
    }

    #[test]
    fn can_build_ref() {
        let mut reader: BinaryHeapQueue<RefEventContainer> = BinaryHeapQueue::with_capacity(16);
        let writer: BinaryHeapQueue<RefEventContainer> = BinaryHeapQueue::default();

        REF_CNT.store(0, AOrdering::SeqCst);

        assert!(reader
            .try_enqueue(0, RefEventContainer::new(&*CLOCK_REF))
            .is_ok());
        let mut sched = SchedExec::new(reader, writer, 0);

        sched.run(0, 4410, &mut ());
        assert_eq!(REF_CNT.load(AOrdering::SeqCst), 0);
        sched.run(1, 4410, &mut ());
        assert_eq!(REF_CNT.load(AOrdering::SeqCst), 1);
    }
}
