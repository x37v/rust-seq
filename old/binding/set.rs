use super::*;

/// A Binding and a value to set it to.
///
/// # Note:
///
/// Used for [`trigger`](../../trigger/index.html)
#[derive(Clone)]
pub enum BindingSet {
    None,
    F32(f32, BindingSetP<f32>),
    I32(i32, BindingSetP<i32>),
    U8(u8, BindingSetP<u8>),
    USize(usize, BindingSetP<usize>),
    Bool(bool, BindingSetP<bool>),
    Midi(MidiValue, BindingSetP<MidiValue>),
    TimeResched(
        crate::time::TimeResched,
        BindingSetP<crate::time::TimeResched>,
    ),
}

impl ParamBindingLatch for BindingSet {
    fn store(&self) {
        match self {
            BindingSet::None => (),
            BindingSet::F32(v, b) => b.set(*v),
            BindingSet::I32(v, b) => b.set(*v),
            BindingSet::U8(v, b) => b.set(*v),
            BindingSet::USize(v, b) => b.set(*v),
            BindingSet::Bool(v, b) => b.set(*v),
            BindingSet::Midi(v, b) => b.set(*v),
            BindingSet::TimeResched(v, b) => b.set(*v),
        }
    }
}

impl Default for BindingSet {
    fn default() -> Self {
        BindingSet::None
    }
}
