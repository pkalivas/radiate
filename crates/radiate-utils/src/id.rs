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
                static COUNTER: ::std::sync::atomic::AtomicU64 =
                    ::std::sync::atomic::AtomicU64::new(1);

                $name(COUNTER.fetch_add(1, ::std::sync::atomic::Ordering::Relaxed))
            }

            pub const fn is_empty(&self) -> bool {
                self.0 == 0
            }

            pub const fn get(&self) -> u64 {
                self.0
            }

            pub fn next(&self) -> Self {
                Self::new()
            }
        }

        #[allow(clippy::from_over_into)]
        impl ::std::convert::Into<u64> for $name {
            fn into(self) -> u64 {
                self.0
            }
        }

        impl ::std::convert::AsRef<u64> for $name {
            fn as_ref(&self) -> &u64 {
                &self.0
            }
        }

        impl ::std::default::Default for $name {
            fn default() -> Self {
                Self::EMPTY
            }
        }

        impl ::std::fmt::Display for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
}
