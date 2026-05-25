mod arithmetic;
mod compare;
pub mod dtype;
mod scalar;
pub mod value;

pub use arithmetic::pow_anyvalue;
pub use dtype::*;
pub use scalar::Scalar;
pub use value::{AnyValue, dedup_slice};
