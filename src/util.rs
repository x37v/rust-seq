use base::TimeSched;
use ptr::ShrPtr;
use std::sync::atomic::{AtomicUsize, Ordering};

pub fn add_clamped(u: usize, i: isize) -> usize {
    if i > 0 {
        u.saturating_add(i as usize)
    } else {
        u.saturating_sub((-i) as usize)
    }
}

pub fn add_atomic_time(current: &ShrPtr<AtomicUsize>, time: &TimeSched) -> usize {
    add_time(current.load(Ordering::SeqCst), time)
}

pub fn add_time(current: usize, time: &TimeSched) -> usize {
    match *time {
        TimeSched::Absolute(t) | TimeSched::ContextAbsolute(t) => t,
        TimeSched::Relative(t) | TimeSched::ContextRelative(t) => add_clamped(current, t),
    }
}
