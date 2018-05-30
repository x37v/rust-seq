extern crate sched;

use sched::{ContextBase, ExecSched, SchedCall, SchedFn, TimeResched};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

pub struct Clock<SrcSnk, Context> {
    period_micros: Arc<AtomicUsize>,
    sched: SchedFn<SrcSnk, Context>,
}

pub struct ClockControl {
    period_micros: Arc<AtomicUsize>,
}

impl ClockControl {
    pub fn set_period(&self, micros: usize) {
        self.period_micros.store(micros, Ordering::SeqCst);
    }
}

pub struct BPMClock {
    clock_tick: usize,
    ticks_per_beat: usize, //AKA PPQ/TPQN
    beats_per_measure: usize,
    beats_per_minute: f32,
}

#[derive(Debug, PartialEq)]
pub struct MeasureBeatTick {
    measure: usize,
    beat: usize,
    tick: usize,
}

impl MeasureBeatTick {
    fn new(measure: usize, beat: usize, tick: usize) -> Self {
        MeasureBeatTick {
            measure,
            beat,
            tick,
        }
    }

    fn measure(&self) -> usize {
        self.measure
    }
    fn beat(&self) -> usize {
        self.beat
    }
    fn tick(&self) -> usize {
        self.tick
    }
}

impl From<(usize, usize, usize)> for MeasureBeatTick {
    fn from(mbt: (usize, usize, usize)) -> Self {
        MeasureBeatTick {
            measure: mbt.0,
            beat: mbt.1,
            tick: mbt.2,
        }
    }
}

impl Into<(usize, usize, usize)> for MeasureBeatTick {
    fn into(self) -> (usize, usize, usize) {
        (self.measure, self.beat, self.tick)
    }
}

impl BPMClock {
    fn new() -> Self {
        BPMClock {
            clock_tick: 0,
            ticks_per_beat: 960,
            beats_per_measure: 4,
            beats_per_minute: 120.0,
        }
    }

    fn set_ticks(&mut self, tick: usize) {
        self.clock_tick = tick;
    }

    fn set_pos(&mut self, pos: MeasureBeatTick) {
        self.clock_tick = pos.measure() * self.ticks_per_beat * self.beats_per_measure
            + pos.beat() * self.ticks_per_beat
            + pos.tick();
    }

    fn bpm(&mut self, value: f32) {
        self.beats_per_minute = value;
    }

    fn ppq(&self) -> usize {
        self.ticks_per_beat
    }

    fn micros_per_tick(&self) -> f32 {
        (60e6 as f64 / (self.ticks_per_beat as f64 * self.beats_per_minute as f64)) as f32
    }

    //measure, beat, tick
    fn pos(&self) -> MeasureBeatTick {
        let ticks_per_measure = self.ticks_per_beat * self.beats_per_measure;
        let measure = self.clock_tick / ticks_per_measure;

        let rem = self.clock_tick - ticks_per_measure * measure;
        let beat = rem / self.ticks_per_beat;

        let tick = rem - beat * self.ticks_per_beat;

        MeasureBeatTick::new(measure, beat, tick)
    }
}

impl<SrcSnk, Context: ContextBase> SchedCall<SrcSnk, Context> for Clock<SrcSnk, Context> {
    fn sched_call(
        &mut self,
        s: &mut ExecSched<SrcSnk, Context>,
        context: &mut Context,
    ) -> TimeResched {
        match self.sched.sched_call(s, context) {
            TimeResched::None => TimeResched::None,
            _ => TimeResched::ContextRelative(std::cmp::max(
                1,
                (self.period_micros.load(Ordering::SeqCst) * context.ticks_per_second())
                    / 1_000_000usize,
            )),
        }
    }
}

impl<SrcSnk, Context> Clock<SrcSnk, Context> {
    pub fn new_micros(period_micros: Arc<AtomicUsize>, sched: SchedFn<SrcSnk, Context>) -> Self {
        Clock {
            period_micros,
            sched,
        }
    }

    pub fn new(period_micros: usize, sched: SchedFn<SrcSnk, Context>) -> (ClockControl, Self) {
        let a = Arc::new(AtomicUsize::new(period_micros));
        (
            ClockControl {
                period_micros: a.clone(),
            },
            Self::new_micros(a, sched),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut clock = BPMClock::new();
        println!("{}", clock.micros_per_tick());

        assert_eq!((0, 0, 0), clock.pos().into());

        clock.set_ticks(960);
        assert_eq!((0, 1, 0), clock.pos().into());

        clock.set_ticks(961);
        assert_eq!((0, 1, 1), clock.pos().into());

        clock.set_ticks(1919);
        assert_eq!((0, 1, 959), clock.pos().into());

        clock.set_ticks(1920);
        assert_eq!((0, 2, 0), clock.pos().into());

        clock.set_ticks(1920 + 960);
        assert_eq!((0, 3, 0), clock.pos().into());

        clock.set_ticks(1920 * 2);
        assert_eq!((1, 0, 0), clock.pos().into());

        clock.set_ticks(1920 * 2 + 1);
        assert_eq!((1, 0, 1), clock.pos().into());

        clock.set_ticks(1920 * 2 + 960 + 2);
        assert_eq!((1, 1, 2), clock.pos().into());

        clock.set_pos((0, 1, 2).into());
        assert_eq!((0, 1, 2), clock.pos().into());

        //overflows
        clock.set_pos((0, 4, 959).into());
        assert_eq!((1, 0, 959), clock.pos().into());

        clock.set_pos((0, 4, 962).into());
        assert_eq!((1, 1, 2), clock.pos().into());

        assert_eq!(MeasureBeatTick::from((1, 1, 2)), clock.pos());
        assert_eq!((1, 1, 2), clock.pos().into());
    }
}
