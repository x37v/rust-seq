//XXX maybe context ticks should have an isize absolute offset?
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TickSched {
    Absolute(usize),
    Relative(isize),
    ContextAbsolute(usize), /* ContextAbsolute(usize, isize) */
    ContextRelative(isize), /* ContextRelative(isize, isize) */
}

/// A forward only time unit.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TickResched {
    Relative(usize),
    ContextRelative(usize), /*ContextRelative(usize, isize) */
    None,
}

pub trait TickContext {
    /// Absolute
    fn tick_now(&self) -> usize;
    fn ticks_per_second(&self) -> usize;

    fn tick_period_micros(&self) -> f32 {
        //XXX likely want to cache this
        1e6f32 / (self.ticks_per_second() as f32)
    }

    /// Context
    fn context_tick_now(&self) -> usize {
        self.tick_now()
    }

    fn context_ticks_per_second(&self) -> usize {
        self.ticks_per_second()
    }

    fn context_tick_period_micros(&self) -> f32 {
        //XXX likely want to cache this
        1e6f32 / (self.context_ticks_per_second() as f32)
    }

    /// Which absolute tick does context 0 happen
    fn context_tick_offset(&self) -> isize {
        0isize
    }

    /// context ticks, base ticks
    fn context_tick_ratio(&self) -> (usize, usize) {
        (self.context_ticks_per_second(), self.ticks_per_second())
    }
}

pub fn offset_tick(tick: usize, offset: isize) -> usize {
    if offset >= 0isize {
        tick.saturating_add(offset as usize)
    } else {
        tick.saturating_sub(-offset as usize)
    }
}

impl TickSched {
    pub fn add<'a>(&self, d: TickResched, context: &'a dyn TickContext) -> Self {
        //XXX update with context math
        let (cratio, bratio) = context.context_tick_ratio();
        match d {
            TickResched::Relative(offset) => match *self {
                TickSched::Absolute(tick) => {
                    TickSched::Absolute(offset_tick(tick, offset as isize))
                }
                TickSched::ContextAbsolute(_ctick) => unimplemented!(),
                TickSched::Relative(aoffset) => TickSched::Relative(offset as isize + aoffset),
                TickSched::ContextRelative(_coffset) => unimplemented!(),
            },
            TickResched::ContextRelative(offset) => match *self {
                TickSched::Absolute(tick) => {
                    let tick = offset_tick(
                        tick,
                        context.context_tick_offset().saturating_add(
                            //XXX conversion to isize could overflow.. figure this out
                            offset.saturating_mul(bratio) as isize / (cratio as isize),
                        ),
                    );
                    TickSched::Absolute(tick)
                }
                TickSched::ContextAbsolute(tick) => {
                    TickSched::ContextAbsolute(offset_tick(tick, offset as isize))
                }
                TickSched::Relative(_offset) => unimplemented!(),
                TickSched::ContextRelative(coffset) => {
                    TickSched::ContextRelative(offset_tick(offset, coffset) as isize)
                }
            },
            TickResched::None => *self,
        }
    }

    pub fn to_absolute<'a>(&self, context: &'a dyn TickContext) -> usize {
        let (cratio, bratio) = context.context_tick_ratio();
        match *self {
            TickSched::Absolute(tick) => tick,
            TickSched::Relative(offset) => offset_tick(context.tick_now(), offset),
            TickSched::ContextAbsolute(tick) => {
                offset_tick(context.tick_now(), context.context_tick_offset())
                    .saturating_add(tick.saturating_mul(bratio) / cratio)
            }
            TickSched::ContextRelative(_offset) => {
                unimplemented!();
                //convert relative to absolute
                //let offset = offset.saturating_mul(ratio.0 as isize) / (div as isize);
                //offset_tick(now, offset).saturating_add(context_tick_offset)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestContext {
        pub tick_now: usize,
        pub ticks_per_second: usize,
        pub context_tick_now: usize,
        pub context_ticks_per_second: usize,
        pub context_tick_offset: isize,
    }

    impl TestContext {
        pub fn new() -> Self {
            Self {
                tick_now: 0,
                ticks_per_second: 44100,
                context_tick_now: 0,
                context_ticks_per_second: 44100,
                context_tick_offset: 0,
            }
        }
    }

    impl TickContext for TestContext {
        /// Absolute
        fn tick_now(&self) -> usize {
            self.tick_now
        }
        fn ticks_per_second(&self) -> usize {
            self.ticks_per_second
        }

        /// Context
        fn context_tick_now(&self) -> usize {
            self.context_tick_now
        }

        fn context_ticks_per_second(&self) -> usize {
            self.context_ticks_per_second
        }

        fn context_tick_offset(&self) -> isize {
            self.context_tick_offset
        }
    }

    #[test]
    fn assert_add_identity() {
        let mut context = TestContext::new();

        let mut tick = TickSched::Absolute(0);
        assert_eq!(tick, tick.add(TickResched::None, &context));
        assert_eq!(tick, tick.add(TickResched::Relative(0), &context));

        tick = TickSched::Absolute(2084);
        assert_eq!(tick, tick.add(TickResched::None, &context));
        assert_eq!(tick, tick.add(TickResched::Relative(0), &context));

        tick = TickSched::ContextAbsolute(0);
        assert_eq!(tick, tick.add(TickResched::None, &context));
        assert_eq!(tick, tick.add(TickResched::ContextRelative(0), &context));

        //offset should have no effect for pure context
        context.context_tick_offset = 1234;
        assert_eq!(tick, tick.add(TickResched::None, &context));
        assert_eq!(tick, tick.add(TickResched::ContextRelative(0), &context));

        tick = TickSched::ContextAbsolute(323);
        assert_eq!(tick, tick.add(TickResched::None, &context));
        assert_eq!(tick, tick.add(TickResched::ContextRelative(0), &context));

        tick = TickSched::ContextRelative(323);
        assert_eq!(tick, tick.add(TickResched::None, &context));
        assert_eq!(tick, tick.add(TickResched::ContextRelative(0), &context));

        tick = TickSched::Relative(32323);
        assert_eq!(tick, tick.add(TickResched::None, &context));
        assert_eq!(tick, tick.add(TickResched::Relative(0), &context));
    }

    #[test]
    fn assert_add_ratio() {
        let mut context = TestContext::new();

        //context 10x faster than base
        context.ticks_per_second = 44100;
        context.context_ticks_per_second = 4410;

        let mut tick = TickSched::Absolute(0);
        assert_eq!(tick, tick.add(TickResched::ContextRelative(0), &context));
        assert_eq!(
            TickSched::Absolute(10),
            tick.add(TickResched::ContextRelative(1), &context)
        );
        assert_eq!(
            TickSched::Absolute(20),
            tick.add(TickResched::ContextRelative(2), &context)
        );

        tick = TickSched::Absolute(1);
        assert_eq!(
            TickSched::Absolute(11),
            tick.add(TickResched::ContextRelative(1), &context)
        );
        assert_eq!(
            TickSched::Absolute(21),
            tick.add(TickResched::ContextRelative(2), &context)
        );

        //context starts before absolute
        context.context_tick_offset = -20;
        tick = TickSched::Absolute(0);
        assert_eq!(
            TickSched::Absolute(0),
            tick.add(TickResched::ContextRelative(0), &context)
        );
        assert_eq!(
            TickSched::Absolute(0),
            tick.add(TickResched::ContextRelative(1), &context)
        );
        assert_eq!(
            TickSched::Absolute(0),
            tick.add(TickResched::ContextRelative(2), &context)
        );
        assert_eq!(
            TickSched::Absolute(10),
            tick.add(TickResched::ContextRelative(3), &context)
        );

        //context 2x slower than base
        context.context_ticks_per_second = 44100 * 2;
        context.context_tick_offset = 0;
        tick = TickSched::Absolute(0);
        assert_eq!(tick, tick.add(TickResched::ContextRelative(0), &context));
        assert_eq!(
            TickSched::Absolute(0),
            tick.add(TickResched::ContextRelative(1), &context)
        );
        assert_eq!(
            TickSched::Absolute(1),
            tick.add(TickResched::ContextRelative(2), &context)
        );

        context.context_tick_offset = 1;
        assert_eq!(
            TickSched::Absolute(1),
            tick.add(TickResched::ContextRelative(0), &context)
        );
        assert_eq!(
            TickSched::Absolute(1),
            tick.add(TickResched::ContextRelative(1), &context)
        );
        assert_eq!(
            TickSched::Absolute(2),
            tick.add(TickResched::ContextRelative(2), &context)
        );

        tick = TickSched::Absolute(100);
        assert_eq!(
            TickSched::Absolute(101),
            tick.add(TickResched::ContextRelative(0), &context)
        );
        assert_eq!(
            TickSched::Absolute(101),
            tick.add(TickResched::ContextRelative(1), &context)
        );
        assert_eq!(
            TickSched::Absolute(102),
            tick.add(TickResched::ContextRelative(2), &context)
        );
    }

    #[test]
    fn to_absolute() {
        //TODO
    }

    #[test]
    fn assert_offset_tick() {
        assert_eq!(0usize, offset_tick(0, -2));
        assert_eq!(0usize, offset_tick(0, 0));
        assert_eq!(0usize, offset_tick(1, -1));
        assert_eq!(0usize, offset_tick(1, -2));
        assert_eq!(0usize, offset_tick(123, -123));
        assert_eq!(0usize, offset_tick(123, -12000));
        assert_eq!(2usize, offset_tick(2, 0));
        assert_eq!(2usize, offset_tick(0, 2));
        assert_eq!(2usize, offset_tick(1, 1));

        assert_eq!(800usize, offset_tick(800, 0));
        assert_eq!(800usize, offset_tick(802, -2));
        assert_eq!(800usize, offset_tick(702, 98));
        assert_eq!(800usize, offset_tick(902, -102));
    }
}
