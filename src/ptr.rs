pub type UniqPtr<T> = Box<T>;
pub type ShrPtr<T> = std::sync::Arc<T>;
pub type SShrPtr<T> = std::sync::Arc<spinlock::Mutex<T>>;
