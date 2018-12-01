use std::sync::Arc;

pub type UniqPtr<T> = Box<T>;
pub type ShrPtr<T> = Arc<T>;
pub type SShrPtr<T> = Arc<spinlock::Mutex<T>>;

#[macro_export]
macro_rules! new_uniqptr {
    ( $x:expr ) => {
        Box::new($x)
    };
}

#[macro_export]
macro_rules! new_shrptr {
    ( $x:expr ) => {
        Arc::new($x)
    };
}

#[macro_export]
macro_rules! new_sshrptr {
    ( $x:expr ) => {
        Arc::new(spinlock::Mutex::new($x))
    };
}
