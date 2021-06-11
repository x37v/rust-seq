use super::*;
use crate::spin::mutex::spin::SpinMutex;

///A packed bool array
pub struct BoolArray<const BYTES: usize> {
    data: SpinMutex<[u8; BYTES]>,
}

impl<const BYTES: usize> BoolArray<BYTES> {
    pub const fn new() -> Self {
        Self {
            data: SpinMutex::new([0; BYTES]),
        }
    }

    pub const fn bytes() -> usize {
        BYTES
    }

    pub fn byte(&self, index: usize) -> Result<u8, ()> {
        if index >= BYTES {
            Err(())
        } else {
            Ok(self.data.lock()[index])
        }
    }

    pub fn toggle(&self, key: usize) -> Result<bool, ()> {
        let byte = key / 8;
        if byte >= BYTES {
            Err(())
        } else {
            let mut g = self.data.lock();
            let bit = key % 8;
            let mask = 1 << bit;
            let cur = g[byte] ^ mask;
            g[byte] = cur;
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
        Self {
            data: SpinMutex::new(bytes),
        }
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
        Self {
            data: SpinMutex::new(bytes),
        }
    }
}

impl<const BYTES: usize> ParamKeyValueGet<bool> for BoolArray<BYTES> {
    fn get_at(&self, key: usize) -> Option<bool> {
        let byte = key / 8;
        if byte >= BYTES {
            None
        } else {
            let bit = key % 8;
            Some(0 != (self.data.lock()[byte] & (1 << bit)))
        }
    }

    fn len(&self) -> Option<usize> {
        Some(BYTES * 8)
    }
}

impl<const BYTES: usize> ParamKeyValueSet<bool> for BoolArray<BYTES> {
    fn set_at(&self, key: usize, value: bool) -> Result<(), bool> {
        let byte = key / 8;
        if byte >= BYTES {
            Err(value)
        } else {
            let mut g = self.data.lock();
            let bit = key % 8;
            g[byte] = (g[byte] & !(1 << bit)) | if value { 1 << bit } else { 0 };
            Ok(())
        }
    }

    fn len(&self) -> Option<usize> {
        Some(BYTES * 8)
    }
}
