use binding::{ParamBinding, SpinlockParamBinding};
use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;

pub type BindingMap = HashMap<String, Box<Any>>;

pub struct BindingCache(pub BindingMap);

impl BindingCache {
    pub fn new() -> Self {
        BindingCache(Default::default())
    }
}

pub trait CacheBindingF32 {
    fn get_f32_binding(&mut self, key: String, default: f32) -> Arc<dyn ParamBinding<f32>>;
}

impl CacheBindingF32 for BindingCache {
    fn get_f32_binding(&mut self, key: String, default: f32) -> Arc<dyn ParamBinding<f32>> {
        if let Some(v) = self.0.get_mut(&key) {
            println!("KEY EXISTS");
            if let Some(b) = v.downcast_mut::<Arc<SpinlockParamBinding<f32>>>() {
                return b.clone();
            } else {
                println!("couldn't get ref {:?}", v);
            }
        }
        let v = Arc::new(SpinlockParamBinding::new(default));
        self.0.insert(key, Box::new(v.clone()));
        v
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
        assert_eq!(x.get(), y.get());

        y.set(53f32);
        assert_eq!(x.get(), y.get());
    }
}
