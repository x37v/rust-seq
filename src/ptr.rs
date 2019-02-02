cfg_if! {
    if #[cfg(feature = "std")] {
        use std::sync::Arc;
    } else if #[cfg(feature = "alloc")] {
        extern crate alloc;
        use alloc::sync::Arc;
        use alloc::boxed::Box;
    }
}

cfg_if! {
    if #[cfg(feature = "alloc")] {

        /// A unique ptr
        pub type UniqPtr<T> = Box<T>;

        /// A shared ptr
        pub type ShrPtr<T> = Arc<T>;

        /// A sync/send mutable shared ptr
        pub type SShrPtr<T> = Arc<spinlock::Mutex<T>>;
    } else {
        /// A unique ptr
        pub type UniqPtr<T> = &'static T;

        /// A shared ptr
        pub type ShrPtr<T> = &'static T;

        /// A sync/send mutable shared ptr
        pub type SShrPtr<T> = &'static spinlock::Mutex<T>;
    }
}
