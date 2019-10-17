//XXX maybe context ticks should have an isize absolute offset?
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TickSched {
    Absolute(usize),
    Relative(isize),
    ContextAbsolute(usize), /* ContextAbsolute(usize, isize) */
    ContextRelative(isize), /* ContextRelative(isize, isize) */
}

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
        self.tick_period_micros()
    }

    /// Which absolute tick does context 0 happen
    fn context_tick_offset(&self) -> isize {
        0isize
    }

    /// context ticks, base ticks
    fn context_tick_ratio(&self) -> (usize, usize) {
        (1usize, 1usize)
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
    pub fn add<'a>(&self, d: TickResched, _context: &'a dyn TickContext) -> Self {
        //XXX update with context math
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
                TickSched::Absolute(_tick) => unimplemented!(),
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
        match *self {
            TickSched::Absolute(tick) => tick,
            TickSched::Relative(offset) => offset_tick(context.tick_now(), offset),
            TickSched::ContextAbsolute(_tick) => {
                unimplemented!();
                //context.tick_now()
                //.saturating_add(tick.saturating_mul(ratio.0) / div)
                //.saturating_add(context_offset)
            }
            TickSched::ContextRelative(_offset) => {
                unimplemented!();
                //convert relative to absolute
                //let offset = offset.saturating_mul(ratio.0 as isize) / (div as isize);
                //offset_tick(now, offset).saturating_add(context_offset)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
