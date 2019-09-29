use crate::event::ticked_value_queue::TickPriorityQueue;
use crate::event::{EventContainer, EventSchedule};
use crate::item_sink::ItemSink;
use crate::time::*;

pub trait SchedulePop<N, T> {
    fn pop_lt(&mut self, index: N) -> Option<(N, T)>;
}

pub struct ScheduleExecutor<R, W, D>
where
    R: SchedulePop<usize, EventContainer>,
    W: TickPriorityQueue<EventContainer>,
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
    schedule: &'a mut dyn TickPriorityQueue<EventContainer>,
}

impl<R, W, D> ScheduleExecutor<R, W, D>
where
    R: SchedulePop<usize, EventContainer>,
    W: TickPriorityQueue<EventContainer>,
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
        while let Some((t, mut event)) = self.schedule_reader.pop_lt(next) {
            //clamp below now, exal and dispose
            let tick = if t < now { now } else { t };
            context.update_tick(tick);
            event.event_eval(&mut context);
            if self.dispose_sink.try_put(event).is_err() {
                //XXX report?
            }
        }

        self.tick_next = next;
    }
}

impl<'a> RootContext<'a> {
    pub fn new(
        tick: usize,
        ticks_per_second: usize,
        schedule: &'a mut dyn TickPriorityQueue<EventContainer>,
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

impl<'a> RootContext<'a> {}
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
        self.schedule.queue(tick, event)
    }
}

impl<'a> TimeContext for RootContext<'a> {
    fn time_now(&self) -> usize {
        self.tick
    }
    fn time_ticks_per_second(&self) -> usize {
        self.ticks_per_second
    }
}
