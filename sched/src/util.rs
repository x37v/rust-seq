pub fn add_clamped(u: usize, i: isize) -> usize {
    if i > 0 {
        u.saturating_add(i as usize)
    } else {
        u.saturating_sub((-i) as usize)
    }
}

pub trait Clamp<T> {
    fn clamp(&self, min: T, max: T) -> T;
}

impl<T> Clamp<T> for T
where
    T: PartialOrd + Copy,
{
    fn clamp(&self, min: T, max: T) -> T {
        if *self < min {
            min
        } else if *self > max {
            max
        } else {
            *self
        }
    }
}
