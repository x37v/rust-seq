extern crate sched;
extern crate spinlock;

use sched::{ContextBase, ExecSched, SchedCall, SchedFn, TimeResched};
use std::cell::Cell;
use std::sync::Arc;

pub struct ParamBinding<T: Copy> {
    lock: spinlock::Mutex<Cell<T>>,
}

impl<T: Copy> ParamBinding<T> {
    pub fn new(value: T) -> Self {
        ParamBinding {
            lock: spinlock::Mutex::new(Cell::new(value)),
        }
    }

    pub fn set(&self, value: T) {
        self.lock.lock().set(value);
    }

    pub fn get(&self) -> T {
        self.lock.lock().get()
    }
}

type FloatBinding = Arc<ParamBinding<f64>>;

pub struct Clock<SrcSnk, Context> {
    tick: usize,
    tick_sub: f64,
    period_micros: FloatBinding,
    sched: SchedFn<SrcSnk, Context>,
}

impl<SrcSnk, Context> Clock<SrcSnk, Context> {
    pub fn new(period_micros: FloatBinding, sched: SchedFn<SrcSnk, Context>) -> Self {
        Clock {
            period_micros,
            sched,
            tick: 0,
            tick_sub: 0f64,
        }
    }
}

impl<SrcSnk, Context> SchedCall<SrcSnk, Context> for Clock<SrcSnk, Context>
where
    Context: ContextBase,
{
    fn sched_call(
        &mut self,
        s: &mut ExecSched<SrcSnk, Context>,
        context: &mut Context,
    ) -> TimeResched {
        if let Some(ticks_per_second) = context.ticks_per_second() {
            assert!(ticks_per_second > 0, "need ticks greater than zero");
            let mut child_context = Context::with_tick(self.tick, context);
            match self.sched.sched_call(s, &mut child_context) {
                TimeResched::None => TimeResched::None,
                _ => {
                    let next = self.tick_sub
                        + (ticks_per_second as f64 * self.period_micros.get()) / 1_000_000f64;
                    self.tick_sub = next.fract();
                    self.tick += 1;
                    //XXX what if next is less than 1?
                    assert!(next >= 1f64, "tick less than sample size not supported");
                    TimeResched::ContextRelative(std::cmp::max(1, next.floor() as usize))
                }
            }
        } else {
            TimeResched::None
        }
    }
}

/*
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
*/

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn param() {
        let x = ParamBinding::new(234);
        assert_eq!(234, x.get());
        x.set(3040);
        assert_eq!(3040, x.get());

        let mut r = Arc::new(ParamBinding::new(234));
        assert_eq!(234, r.get());
        r.set(3040);
        assert_eq!(3040, r.get());

        //clone gets updates
        let mut c = r.clone();
        assert_eq!(3040, c.get());
        r.set(3041);
        assert_eq!(3041, c.get());
        assert_eq!(3041, r.get());

        let child = thread::spawn(move || {
            c.set(2084);
        });
        assert!(child.join().is_ok());
        assert_eq!(2084, r.get());
    }

    #[test]
    fn it_works() {
        /*
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

        clock.set_pos(&((0, 1, 2).into()));
        assert_eq!((0, 1, 2), clock.pos().into());

        //overflows
        clock.set_pos(&((0, 4, 959).into()));
        assert_eq!((1, 0, 959), clock.pos().into());

        clock.set_pos(&((0, 4, 962).into()));
        assert_eq!((1, 1, 2), clock.pos().into());

        assert_eq!(MeasureBeatTick::from((1, 1, 2)), clock.pos());
        assert_eq!((1, 1, 2), clock.pos().into());
        */
    }
}
