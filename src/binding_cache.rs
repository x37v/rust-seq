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

    pub fn get<T>(&mut self, key: String, default: T) -> Result<Arc<dyn ParamBinding<T>>, GetError>
    where
        T: Send + Copy + 'static,
    {
        if let Some(v) = self.0.get_mut(&key) {
            if let Some(b) = v.downcast_mut::<Arc<SpinlockParamBinding<T>>>() {
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
    fn cache() {
        let mut c = BindingCache::new();
        let x = c.get::<f32>("soda".to_string(), 43f32);
        let y = c.get::<f32>("soda".to_string(), 12f32);
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
        assert!(c.get::<f32>("foo".to_string(), 23f32).is_err());

        let y = c.get::<f32>("soda".to_string(), 12f32);
        let yr = y.unwrap();
        assert_eq!(53f32, xr.get());
        assert_eq!(53f32, yr.get());

        let v = c.get::<f32>("soda".to_string(), 1f32);
        assert!(v.is_ok());
        let v = v.unwrap();
        assert_eq!(53f32, v.get());
    }
}
