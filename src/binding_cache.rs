use binding::{ParamBinding, SpinlockParamBinding};
use failure::Fail;
use std::any::Any;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicIsize, AtomicUsize};
use std::sync::Arc;

#[derive(Debug, Fail)]
#[fail(display = "entry exists but type is wrong: {}", key)]
pub struct GetError {
    key: String,
}

pub type BindingMap = HashMap<String, Box<Any>>;

pub struct BindingCache(pub BindingMap);

impl BindingCache {
    pub fn new() -> Self {
        BindingCache(Default::default())
    }

    fn get_item<T, U>(
        &mut self,
        key: String,
        default: U,
    ) -> Result<Arc<dyn ParamBinding<U>>, GetError>
    where
        T: Default + ParamBinding<U> + 'static,
    {
        if let Some(v) = self.0.get_mut(&key) {
            if let Some(b) = v.downcast_mut::<Arc<T>>() {
                Ok(b.clone())
            } else {
                Err(GetError { key: key })
            }
        } else {
            let v: Arc<T> = Arc::new(Default::default());
            v.set(default);

            let b: Box<Arc<T>> = Box::new(v.clone());
            self.0.insert(key, b);
            Ok(v)
        }
    }

    pub fn get_usize(
        &mut self,
        key: String,
        default: usize,
    ) -> Result<Arc<dyn ParamBinding<usize>>, GetError> {
        self.get_item::<AtomicUsize, usize>(key, default)
    }

    pub fn get_isize(
        &mut self,
        key: String,
        default: isize,
    ) -> Result<Arc<dyn ParamBinding<isize>>, GetError> {
        self.get_item::<AtomicIsize, isize>(key, default)
    }

    pub fn get_bool(
        &mut self,
        key: String,
        default: bool,
    ) -> Result<Arc<dyn ParamBinding<bool>>, GetError> {
        self.get_item::<AtomicBool, bool>(key, default)
    }

    pub fn get_spinlock<T>(
        &mut self,
        key: String,
        default: T,
    ) -> Result<Arc<dyn ParamBinding<T>>, GetError>
    where
        T: Send + Copy + Default + 'static,
    {
        self.get_item::<SpinlockParamBinding<T>, T>(key, default)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use binding::bpm::{
        Clock, ClockBPMBinding, ClockData, ClockPPQBinding, ClockPeriodMicroBinding,
    };

    #[test]
    fn cache() {
        let mut c = BindingCache::new();
        let x = c.get_spinlock::<f32>("soda".to_string(), 43f32);
        let y = c.get_spinlock::<f32>("soda".to_string(), 12f32);
        assert!(x.is_ok());
        assert!(y.is_ok());

        let xr = x.unwrap();
        let yr = y.unwrap();

        assert_eq!(43f32, xr.get());
        assert_eq!(43f32, yr.get());

        yr.set(53f32);
        assert_eq!(53f32, xr.get());
        assert_eq!(53f32, yr.get());

        c.0.insert("foo".to_string(), Box::new(3));
        assert!(c.get_spinlock::<f32>("foo".to_string(), 23f32).is_err());

        let y = c.get_spinlock::<f32>("soda".to_string(), 12f32);
        let yr = y.unwrap();
        assert_eq!(53f32, xr.get());
        assert_eq!(53f32, yr.get());

        let v = c.get_spinlock::<f32>("soda".to_string(), 1f32);
        assert!(v.is_ok());
        let v = v.unwrap();
        assert_eq!(53f32, v.get());

        assert!(c.get_spinlock::<f64>("soda".to_string(), 1f64).is_err());
        assert!(c.get_usize("soda".to_string(), 1).is_err());
        assert!(c.get_isize("soda".to_string(), 1).is_err());
        assert!(c.get_bool("soda".to_string(), true).is_err());
    }

    /*
    #[test]
    fn cache_bpm() {
        let mut c = BindingCache::new();
        let f = c.get::<f32>("soda".to_string(), 43f32);
        let b = c.get::<::binding::bpm::ClockData>(
            "bpm".to_string(),
            ::binding::bpm::ClockData::new(110f32, 990),
        );
    
        assert!(f.is_ok());
        assert!(b.is_ok());
        let f = f.unwrap();
        let b = b.unwrap();
        assert_eq!(43f32, f.get());
        assert_eq!(110f32, b.get().bpm());
        assert_eq!(990, b.get().ppq());
    
        let b2 = c.get::<ClockData>("bpm".to_string(), ClockData::new(1f32, 10));
        assert!(b2.is_ok());
        let b2 = b2.unwrap();
    
        let bpm = Arc::new(ClockBPMBinding(b2.clone()));
        let ppq = Arc::new(ClockPPQBinding(b2.clone()));
        let micros = Arc::new(ClockPeriodMicroBinding(b2.clone()));
    }
    */
}
