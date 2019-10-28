use super::*;

use core::sync::atomic::*;

macro_rules! impl_get_set {
    ($t:ty, $a:ty) => {
        impl ParamBindingGet<$t> for $a {
            fn get(&self) -> $t {
                self.load(Ordering::SeqCst)
            }
        }
        impl ParamBindingSet<$t> for $a {
            fn set(&self, value: $t) {
                self.store(value, Ordering::SeqCst);
            }
        }
    };
}

impl_get_set!(bool, AtomicBool);
impl_get_set!(i8, AtomicI8);
impl_get_set!(i16, AtomicI16);
impl_get_set!(i32, AtomicI32);
impl_get_set!(isize, AtomicIsize);
impl_get_set!(u8, AtomicU8);
impl_get_set!(u16, AtomicU16);
impl_get_set!(u32, AtomicU32);
impl_get_set!(usize, AtomicUsize);

//requires 64bit pointer size, should be able to enable for other 64bit targets..
#[cfg(target_arch = "x86_64")]
impl_get_set!(i64, AtomicI64);
#[cfg(target_arch = "x86_64")]
impl_get_set!(u64, AtomicU64);
