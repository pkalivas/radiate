pub mod executor;
pub mod math;
pub mod random_provider;
pub mod sync;
pub mod tracker;

pub use executor::Executor;
pub use math::SubsetMode;
pub use math::subset;
pub use random_provider::RdRand;
pub use sync::{CommandChannel, MutCell, WaitGroup, WaitGuard, get_thread_pool};

#[macro_export]
macro_rules! sentry_id {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        #[repr(transparent)]
        pub struct $name(u64);

        impl $name {
            pub const EMPTY: Self = $name(0);

            pub fn new() -> Self {
                use std::sync::atomic::Ordering;

                static COUNTER: AtomicU64 = AtomicU64::new(1);
                $name(COUNTER.fetch_add(1, Ordering::Relaxed))
            }
        }

        impl Into<u64> for $name {
            fn into(self) -> u64 {
                self.0
            }
        }

        impl AsRef<u64> for $name {
            fn as_ref(&self) -> &u64 {
                &self.0
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::EMPTY
            }
        }
    };
}
