use crate::event::*;
use crate::item_sink::ItemSink;
use crate::pqueue::{TickPriorityDequeue, TickPriorityEnqueue};
use crate::time::*;

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

pub struct RootContext<'a> {
    tick: usize,
    ticks_per_second: usize,
    schedule: &'a mut dyn TickPriorityEnqueue<EventContainer>,
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
                TimeResched::Relative(t) => Some(TimeSched::Relative(t as isize)),
                TimeResched::ContextRelative(t) => Some(TimeSched::ContextRelative(t as isize)),
                TimeResched::None => None,
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

impl<'a> RootContext<'a> {
    pub fn new(
        tick: usize,
        ticks_per_second: usize,
        schedule: &'a mut dyn TickPriorityEnqueue<EventContainer>,
    ) -> Self {
        Self {
            tick,
            ticks_per_second,
            schedule,
        }
    }

    pub fn update_tick(&mut self, tick: usize) {
        self.tick = tick;
    }
}

impl<'a> EventSchedule for RootContext<'a> {
    fn event_schedule(
        &mut self,
        time: TimeSched,
        event: EventContainer,
    ) -> Result<(), EventContainer> {
        //in the root, context and absolute are the same
        let tick = match time {
            TimeSched::Absolute(t) | TimeSched::ContextAbsolute(t) => t,
            TimeSched::Relative(o) | TimeSched::ContextRelative(o) => offset_tick(self.tick, o),
        };
        self.schedule.enqueue(tick, event)
    }
}

impl<'a> TickContext for RootContext<'a> {
    fn tick_now(&self) -> usize {
        self.tick
    }
    fn ticks_per_second(&self) -> usize {
        self.ticks_per_second
    }
}
