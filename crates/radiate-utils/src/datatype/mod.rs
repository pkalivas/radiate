mod arithmetic;
mod compare;
pub mod dtype;
mod scalar;
pub mod value;

use std::time::Duration;

pub use arithmetic::pow_anyvalue;
pub use dtype::*;
pub use scalar::Scalar;
pub use value::{AnyValue, dedup_slice};

macro_rules! impl_dtype {
    ($t:ty, $dtype:expr) => {
        impl DType for $t {
            fn dtype(&self) -> DataType {
                $dtype
            }
        }
    };
}

impl_dtype!(u8, DataType::UInt8);
impl_dtype!(u16, DataType::UInt16);
impl_dtype!(u32, DataType::UInt32);
impl_dtype!(u64, DataType::UInt64);
impl_dtype!(u128, DataType::UInt128);

impl_dtype!(i8, DataType::Int8);
impl_dtype!(i16, DataType::Int16);
impl_dtype!(i32, DataType::Int32);
impl_dtype!(i64, DataType::Int64);
impl_dtype!(i128, DataType::Int128);

impl_dtype!(f32, DataType::Float32);
impl_dtype!(f64, DataType::Float64);

impl_dtype!(usize, DataType::Usize);

impl_dtype!(Duration, DataType::Duration);

impl_dtype!(bool, DataType::Boolean);

impl_dtype!(char, DataType::Char);
impl_dtype!(String, DataType::String);
