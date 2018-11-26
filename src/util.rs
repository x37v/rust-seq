pub fn add_clamped(u: usize, i: isize) -> usize {
    if i > 0 {
        u.saturating_add(i as usize)
    } else {
        u.saturating_sub((-i) as usize)
    }
}
