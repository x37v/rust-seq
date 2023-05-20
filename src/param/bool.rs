use super::*;
use core::{
    mem::MaybeUninit,
    sync::atomic::{AtomicU8, Ordering},
};

///A packed bool array
///NOTE this expects that you don't alter data from multiple threads, but you can read it from
///multiple
pub struct BoolArray<const BYTES: usize> {
    data: [AtomicU8; BYTES],
}

impl<const BYTES: usize> BoolArray<BYTES> {
    pub const fn new() -> Self {
        let mut data: [AtomicU8; BYTES] = unsafe { MaybeUninit::uninit().assume_init() };
        //for isn't const safe
        let mut i = 0;
        while i < BYTES {
            data[i] = AtomicU8::new(0);
            i += 1;
        }
        Self { data }
    }

    pub const fn bytes() -> usize {
        BYTES
    }

    pub fn byte(&self, index: usize) -> Result<u8, ()> {
        self.data
            .get(index)
            .map(|v| v.load(Ordering::SeqCst))
            .ok_or(())
    }

    pub fn toggle(&self, key: usize) -> Result<bool, ()> {
        let byte = key / 8;
        if byte >= BYTES {
            Err(())
        } else {
            let bit = key % 8;
            let mask = 1 << bit;
            let cur = self.byte(byte).unwrap() ^ mask;
            self.data[byte].store(cur, Ordering::SeqCst);
            Ok(cur & mask != 0)
        }
    }
}

impl<const BYTES: usize> Default for BoolArray<BYTES> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const BYTES: usize> From<[u8; BYTES]> for BoolArray<BYTES> {
    fn from(bytes: [u8; BYTES]) -> Self {
        let mut v: Self = Self::new();
        for (o, i) in v.data.iter_mut().zip(bytes) {
            o.store(i, Ordering::SeqCst);
        }
        v
    }
}

impl<const BYTES: usize> From<&[bool]> for BoolArray<BYTES> {
    fn from(values: &[bool]) -> Self {
        assert!(values.len() <= BYTES * 8);
        let mut bytes = [0; BYTES];
        for (index, v) in values.iter().enumerate() {
            if *v {
                bytes[index / 8] |= 1 << (index % 8);
            }
        }
        Self::from(bytes)
    }
}

impl<U, const BYTES: usize> ParamKeyValueGet<bool, U> for BoolArray<BYTES> {
    fn get_at(&self, key: usize, _user_data: &mut U) -> Option<bool> {
        let byte = key / 8;
        self.byte(byte)
            .ok()
            .map(|cur| (cur & (1 << (key % 8))) != 0)
    }

    fn len(&self, _user_data: &mut U) -> Option<usize> {
        Some(BYTES * 8)
    }
}

impl<U, const BYTES: usize> ParamKeyValueSet<bool, U> for BoolArray<BYTES> {
    fn set_at(&self, key: usize, value: bool, _user_data: &mut U) -> Result<(), bool> {
        let byte = key / 8;
        if byte >= BYTES {
            Err(value)
        } else {
            let cur = self.data[byte].load(Ordering::SeqCst);
            let bit = key % 8;
            self.data[byte].store(
                (cur & !(1 << bit)) | if value { 1 << bit } else { 0 },
                Ordering::SeqCst,
            );
            Ok(())
        }
    }

    fn len(&self, _user_data: &mut U) -> Option<usize> {
        Some(BYTES * 8)
    }
}
