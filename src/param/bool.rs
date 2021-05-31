use super::*;
use crate::spin::mutex::spin::SpinMutex;

///A packed bool array
pub struct BoolArray<const BYTES: usize> {
    data: SpinMutex<[u8; BYTES]>,
}

impl<const BYTES: usize> BoolArray<BYTES> {
    pub fn new() -> Self {
        Self {
            data: SpinMutex::new([0; BYTES]),
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
