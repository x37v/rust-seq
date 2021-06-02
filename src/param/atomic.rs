use super::*;
use core::sync::atomic::*;

macro_rules! impl_get_set {
    ($t:ty, $a:ty) => {
        impl ParamGet<$t> for $a {
            fn get(&self) -> $t {
                self.load(Ordering::SeqCst)
            }
        }
        impl ParamSet<$t> for $a {
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

//requires 64bit pointer size, will not work on some thumb/arm targets
#[cfg(target_pointer_width = "64")]
impl_get_set!(i64, AtomicI64);
#[cfg(target_pointer_width = "64")]
impl_get_set!(u64, AtomicU64);

/*
impl<T> ParamGet<T> for crate::atomic::Atomic<T>
where
    T: Copy + Send,
{
    fn get(&self) -> T {
        self.load(Ordering::SeqCst)
    }
}

impl<T> ParamSet<T> for crate::atomic::Atomic<T>
where
    T: Copy + Send,
{
    fn set(&self, value: T) {
        self.store(value, Ordering::SeqCst);
    }
}
*/
