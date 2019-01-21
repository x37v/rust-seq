cfg_if! {
    if #[cfg(feature = "std")] {
        /// A unique ptr
        pub type UniqPtr<T> = Box<T>;

        /// A shared ptr
        pub type ShrPtr<T> = std::sync::Arc<T>;

        /// A sync/send mutable shared ptr
        pub type SShrPtr<T> = std::sync::Arc<spinlock::Mutex<T>>;
    } else {
        /// A unique ptr
        pub type UniqPtr<T> = &'static T;

        /// A shared ptr
        pub type ShrPtr<T> = &'static T;

        /// A sync/send mutable shared ptr
        pub type SShrPtr<T> = &'static mut ::spinlock::Mutex<T>;
    }
}
