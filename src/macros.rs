#![macro_use]

#[macro_export]
macro_rules! new_uniqptr {
    ( $x:expr ) => {
        Box::new($x)
    };
}

#[macro_export]
macro_rules! new_shrptr {
    ( $x:expr ) => {
        std::sync::Arc::new($x)
    };
}

#[macro_export]
macro_rules! new_sshrptr {
    ( $x:expr ) => {
        std::sync::Arc::new(spinlock::Mutex::new($x))
    };
}
