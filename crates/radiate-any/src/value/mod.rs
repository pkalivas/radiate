mod arithmetic;
mod dtype;
mod field;
mod scalar;
mod value;

pub use arithmetic::mean_anyvalue;
pub use dtype::{DataType, dtype_names};
pub use field::Field;
pub use scalar::Scalar;
pub use value::{AnyValue, apply_zipped_slice, apply_zipped_struct_slice};
