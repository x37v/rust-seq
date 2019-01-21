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
