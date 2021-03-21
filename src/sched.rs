use crate::context::RootContext;
use crate::event::*;
use crate::pqueue::{TickPriorityDequeue, TickPriorityEnqueue};
use crate::tick::*;

pub struct ScheduleExecutor<R, W, E>
where
    R: TickPriorityDequeue<E>,
    W: TickPriorityEnqueue<E>,
{
    tick_next: usize,
    schedule_reader: R,
    schedule_writer: W,
    _phantom: core::marker::PhantomData<fn() -> E>,
}

impl<R, W, E> ScheduleExecutor<R, W, E>
where
    R: TickPriorityDequeue<E>,
    W: TickPriorityEnqueue<E>,
    E: EventEval<E>,
{
    pub fn new(schedule_reader: R, schedule_writer: W) -> Self {
        Self {
            tick_next: 0usize,
            schedule_reader,
            schedule_writer,
            _phantom: Default::default(),
        }
    }

    pub fn run(&mut self, ticks: usize, ticks_per_second: usize) {
        let now = self.tick_next;
        let next = now + ticks;
        let mut context = RootContext::new(now, ticks_per_second, &mut self.schedule_writer);

        //evaluate events before next
        while let Some((t, mut event)) = self.schedule_reader.dequeue_lt(next) {
            //clamp below now, exal and dispose
            let tick = if t < now { now } else { t };
            context.update_tick(tick);

            //eval and see about rescheduling
            let r = match event.event_eval(&mut context) {
                TickResched::Relative(t) => Some(TickSched::Relative(t as isize)),
                TickResched::ContextRelative(t) => Some(TickSched::ContextRelative(t as isize)),
                TickResched::None => None,
            };

            //try to reschedule if we should
            if let Some(t) = r {
                let _ = context.event_try_schedule(t, event);
            }
        }

        self.tick_next = next;
    }

    pub fn tick_next(&self) -> usize {
        self.tick_next
    }
}

#[cfg(all(test, feature = "with_alloc", feature = "std"))]
mod tests {
    use super::*;
    use crate::{
        event::{boxed::EventContainer, EventEval, EventEvalContext},
        pqueue::binaryheap::BinaryHeapQueue,
    };

    #[derive(PartialOrd, Ord, PartialEq, Eq)]
    enum EnumEvent {
        A,
    }

    impl EventEval<EnumEvent> for EnumEvent {
        fn event_eval(&mut self, _context: &mut dyn EventEvalContext<EnumEvent>) -> TickResched {
            TickResched::None
        }
    }

    #[test]
    fn can_build_boxed() {
        let reader: BinaryHeapQueue<EventContainer> = BinaryHeapQueue::with_capacity(16);
        let writer: BinaryHeapQueue<EventContainer> = BinaryHeapQueue::default();
        let mut sched = ScheduleExecutor::new(reader, writer);
        sched.run(0, 16);
    }

    #[test]
    fn can_build_enum() {
        let reader: BinaryHeapQueue<EnumEvent> = BinaryHeapQueue::with_capacity(16);
        let writer: BinaryHeapQueue<EnumEvent> = BinaryHeapQueue::default();
        let mut sched = ScheduleExecutor::new(reader, writer);
        sched.run(0, 16);
    }
}
