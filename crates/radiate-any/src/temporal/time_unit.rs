use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Hash, Serialize, Deserialize)]
pub enum TimeUnit {
    Nanoseconds,
    Microseconds,
    Milliseconds,
}

impl std::fmt::Display for TimeUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TimeUnit::Nanoseconds => {
                write!(f, "ns")
            }
            TimeUnit::Microseconds => {
                write!(f, "μs")
            }
            TimeUnit::Milliseconds => {
                write!(f, "ms")
            }
        }
    }
}

impl TimeUnit {
    pub fn to_ascii(self) -> &'static str {
        use TimeUnit::*;
        match self {
            Nanoseconds => "ns",
            Microseconds => "us",
            Milliseconds => "ms",
        }
    }
}
