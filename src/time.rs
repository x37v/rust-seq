//XXX maybe context times should have an isize absolute offset?
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TimeSched {
    Absolute(usize),
    Relative(isize),
    ContextAbsolute(usize), /* ContextAbsolute(usize, isize) */
    ContextRelative(isize), /* ContextRelative(isize, isize) */
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TimeResched {
    Relative(usize),
    ContextRelative(usize), /*ContextRelative(usize, isize) */
    None,
}

pub trait TimeContext {
    /// Absolute
    fn time_now(&self) -> usize;
    fn time_ticks_per_second(&self) -> usize;
    /// Which absolute tick does context 0 happen
    fn time_context_offset(&self) -> usize {
        0usize
    }
    /// context ticks, base ticks
    fn time_context_tick_ratio(&self) -> (usize, usize) {
        (1usize, 1usize)
    }
}

fn offset_tick(tick: usize, offset: isize) -> usize {
    if offset >= 0isize {
        tick.saturating_add(offset as usize)
    } else {
        tick.saturating_sub(-offset as usize)
    }
}

impl TimeSched {
    pub fn add(&self, d: TimeResched, now: usize, ratio: (usize, usize)) -> Self {
        let offset = match d {
            TimeResched::Relative(offset) => offset,
            TimeResched::ContextRelative(offset) => offset, //TODO context math?
            TimeResched::None => 0usize,
        } as isize;
        //TODO context math?
        match *self {
            TimeSched::Absolute(tick) => TimeSched::Absolute(offset_tick(tick, offset)),
            TimeSched::ContextAbsolute(tick) => TimeSched::Absolute(offset_tick(tick, offset)),
            TimeSched::Relative(now_offset) | TimeSched::ContextRelative(now_offset) => {
                TimeSched::Absolute(offset_tick(now, now_offset.saturating_add(offset)))
            }
        }
    }

    /// now: absolute ticks
    /// context_offset: the absolute tick that context: 0 starts
    /// ratio: (context ticks, absolute ticks)
    pub fn to_absolute(&self, now: usize, context_offset: usize, ratio: (usize, usize)) -> usize {
        let div = if ratio.1 == 0 { 1 } else { ratio.1 };
        //TODO TEST!!!
        match *self {
            TimeSched::Absolute(tick) => tick,
            TimeSched::Relative(offset) => offset_tick(now, offset),
            TimeSched::ContextAbsolute(tick) => now
                .saturating_add(tick.saturating_mul(ratio.0) / div)
                .saturating_add(context_offset),
            TimeSched::ContextRelative(offset) => {
                //convert relative to absolute
                let offset = offset.saturating_mul(ratio.0 as isize) / (div as isize);
                offset_tick(now, offset).saturating_add(context_offset)
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
