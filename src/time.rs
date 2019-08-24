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
    fn now(&self) -> usize;
    fn ticks_per_second(&self) -> usize;
    /// context ticks, base ticks
    fn context_tick_ratio(&self) -> (usize, usize) {
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
    pub fn to_absolute(&self, now: usize, ratio: (usize, usize)) -> usize {
        //TODO context math
        match *self {
            TimeSched::Absolute(tick) | TimeSched::ContextAbsolute(tick) => tick,
            TimeSched::Relative(offset) | TimeSched::ContextRelative(offset) => {
                offset_tick(now, offset)
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
