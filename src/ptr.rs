/// A unique ptr
pub type UniqPtr<T> = Box<T>;

/// A shared ptr
pub type ShrPtr<T> = std::sync::Arc<T>;

/// A sync/send mutable shared ptr
pub type SShrPtr<T> = std::sync::Arc<spinlock::Mutex<T>>;
