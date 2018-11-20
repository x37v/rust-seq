use binding::{ParamBinding, SpinlockParamBinding};
use failure::Fail;
use std::any::Any;
use std::collections::HashMap;
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
}

pub trait CacheBindingF32 {
    fn get_f32_binding(
        &mut self,
        key: String,
        default: f32,
    ) -> Result<Arc<dyn ParamBinding<f32>>, GetError>;
}

impl CacheBindingF32 for BindingCache {
    fn get_f32_binding(
        &mut self,
        key: String,
        default: f32,
    ) -> Result<Arc<dyn ParamBinding<f32>>, GetError> {
        if let Some(v) = self.0.get_mut(&key) {
            if let Some(b) = v.downcast_mut::<Arc<SpinlockParamBinding<f32>>>() {
                Ok(b.clone())
            } else {
                Err(GetError { key: key })
            }
        } else {
            let v = Arc::new(SpinlockParamBinding::new(default));
            self.0.insert(key, Box::new(v.clone()));
            Ok(v)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blah() {
        let mut c = BindingCache::new();
        let x = c.get_f32_binding("soda".to_string(), 43f32);
        let y = c.get_f32_binding("soda".to_string(), 12f32);
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
        assert!(c.get_f32_binding("foo".to_string(), 23f32).is_err());

        let y = c.get_f32_binding("soda".to_string(), 12f32);
        let yr = y.unwrap();
        assert_eq!(53f32, xr.get());
        assert_eq!(53f32, yr.get());
    }
}
