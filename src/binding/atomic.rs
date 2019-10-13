use super::*;

use core::sync::atomic::*;

macro_rules! impl_set {
    ($t:ty, $a:ty) => {
        impl ParamBindingSet<$t> for $a {
            fn set(&self, value: $t) {
                self.store(value, Ordering::SeqCst);
            }
        }
    };
}

impl_set!(bool, AtomicBool);
impl_set!(i8, AtomicI8);
impl_set!(i16, AtomicI16);
impl_set!(i32, AtomicI32);
impl_set!(i64, AtomicI64);
impl_set!(isize, AtomicIsize);
impl_set!(u8, AtomicU8);
impl_set!(u16, AtomicU16);
impl_set!(u32, AtomicU32);
impl_set!(u64, AtomicU64);
impl_set!(usize, AtomicUsize);
