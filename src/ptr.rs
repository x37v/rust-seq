cfg_if! {
    if #[cfg(feature = "std")] {
        /// A unique ptr
        pub type UniqPtr<T> = Box<T>;

        /// A shared ptr
        pub type ShrPtr<T> = std::sync::Arc<T>;

        /// A sync/send mutable shared ptr
        pub type SShrPtr<T> = std::sync::Arc<spinlock::Mutex<T>>;
    } else {
        extern crate alloc;

        struct Src<T> { //static, fake reference counted
            data: &'static mut T
        }

        impl<T> Clone for Src<T> {
            fn clone(&self) -> Src<T> {
                Src { data: self.data }
            }
        }

        /// A unique ptr
        pub type UniqPtr<T> = alloc::boxed::Box<T>;

        /// A shared ptr
        pub type ShrPtr<T> = alloc::sync::Arc<T>;

        /// A sync/send mutable shared ptr
        pub type SShrPtr<T> = alloc::sync::Arc<spinlock::Mutex<T>>;
    }
}
