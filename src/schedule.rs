use crate::context::RootContext;
use crate::event::*;
use crate::item_sink::ItemSink;
use crate::pqueue::{TickPriorityDequeue, TickPriorityEnqueue};
use crate::tick::*;

pub struct ScheduleExecutor<R, W, D>
where
    R: TickPriorityDequeue<EventContainer>,
    W: TickPriorityEnqueue<EventContainer>,
    D: ItemSink<EventContainer>,
{
    tick_next: usize,
    schedule_reader: R,
    schedule_writer: W,
    dispose_sink: D,
}

impl<R, W, D> ScheduleExecutor<R, W, D>
where
    R: TickPriorityDequeue<EventContainer>,
    W: TickPriorityEnqueue<EventContainer>,
    D: ItemSink<EventContainer>,
{
    pub fn new(dispose_sink: D, schedule_reader: R, schedule_writer: W) -> Self {
        Self {
            tick_next: 0usize,
            dispose_sink,
            schedule_reader,
            schedule_writer,
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

            //try to reschedule if we should, otherwise dispose
            let e = if let Some(t) = r {
                match context.event_schedule(t, event) {
                    Ok(()) => Ok(()),
                    Err(e) => {
                        //XXX report, error re-scheduling
                        self.dispose_sink.try_put(e)
                    }
                }
            } else {
                self.dispose_sink.try_put(event)
            };
            if e.is_err() {
                //XXX report, error disposing
            }
        }

        self.tick_next = next;
    }

    pub fn tick_next(&self) -> usize {
        self.tick_next
    }
}
