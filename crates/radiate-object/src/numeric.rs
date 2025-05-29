use crate::AnyValue;
use std::ops::{Add, Div, Mul, Sub};

impl<'a> AnyValue<'a> {
    pub fn from_numeric<T: Into<AnyValue<'a>>>(value: T) -> Self {
        value.into()
    }
}

pub trait NumericOp {
    type Output;
    fn numeric_op(self, other: Self, op: impl FnOnce(f64, f64) -> f64) -> Self::Output;
}

pub trait ToF64 {
    fn to_f64(self) -> f64;
}

macro_rules! impl_to_f64 {
    ($($t:ty),*) => {
        $(
            impl ToF64 for $t {
                fn to_f64(self) -> f64 {
                    self as f64
                }
            }
        )*
    };
}

impl_to_f64!(u8, u16, u32, u64, i8, i16, i32, i64, i128, f32, f64);

// Helper function to convert AnyValue to f64
fn to_f64(value: &AnyValue<'_>) -> Option<f64> {
    match value {
        AnyValue::UInt8(v) => Some(v.to_f64()),
        AnyValue::UInt16(v) => Some(v.to_f64()),
        AnyValue::UInt32(v) => Some(v.to_f64()),
        AnyValue::UInt64(v) => Some(v.to_f64()),
        AnyValue::Int8(v) => Some(v.to_f64()),
        AnyValue::Int16(v) => Some(v.to_f64()),
        AnyValue::Int32(v) => Some(v.to_f64()),
        AnyValue::Int64(v) => Some(v.to_f64()),
        AnyValue::Int128(v) => Some(v.to_f64()),
        AnyValue::Float32(v) => Some(v.to_f64()),
        AnyValue::Float64(v) => Some(v.to_f64()),
        _ => None,
    }
}

// Helper function to convert f64 back to appropriate AnyValue type
fn from_f64<'a>(value: f64, target_type: &AnyValue<'_>) -> AnyValue<'a> {
    if value.is_nan() || value.is_infinite() {
        return AnyValue::Null;
    }

    match target_type {
        AnyValue::UInt8(_) => AnyValue::UInt8(value as u8),
        AnyValue::UInt16(_) => AnyValue::UInt16(value as u16),
        AnyValue::UInt32(_) => AnyValue::UInt32(value as u32),
        AnyValue::UInt64(_) => AnyValue::UInt64(value as u64),
        AnyValue::Int8(_) => AnyValue::Int8(value as i8),
        AnyValue::Int16(_) => AnyValue::Int16(value as i16),
        AnyValue::Int32(_) => AnyValue::Int32(value as i32),
        AnyValue::Int64(_) => AnyValue::Int64(value as i64),
        AnyValue::Int128(_) => AnyValue::Int128(value as i128),
        AnyValue::Float32(_) => AnyValue::Float32(value as f32),
        AnyValue::Float64(_) => AnyValue::Float64(value),
        _ => AnyValue::Null,
    }
}

impl NumericOp for AnyValue<'_> {
    type Output = Self;

    fn numeric_op(self, other: Self, op: impl FnOnce(f64, f64) -> f64) -> Self::Output {
        if !self.is_numeric() || !other.is_numeric() {
            return self;
        }

        match (to_f64(&self), to_f64(&other)) {
            (Some(a), Some(b)) => from_f64(op(a, b), &self),
            _ => AnyValue::Null,
        }
    }
}

macro_rules! impl_numeric_op {
    ($trait:ident, $fn:ident, $op:tt) => {
        impl $trait for AnyValue<'_> {
            type Output = Self;

            fn $fn(self, other: Self) -> Self::Output {
                self.numeric_op(other, |a, b| a $op b)
            }
        }
    };
}

impl_numeric_op!(Add, add, +);
impl_numeric_op!(Sub, sub, -);
impl_numeric_op!(Mul, mul, *);
impl_numeric_op!(Div, div, /);

#[cfg(test)]
mod tests {
    use crate::AnyValue;

    #[test]
    fn test_numeric_operations() {
        let a = AnyValue::Int32(10);
        let b = AnyValue::Int32(5);

        assert_eq!(a.clone() + b.clone(), AnyValue::Int32(15));
        assert_eq!(a.clone() - b.clone(), AnyValue::Int32(5));
        assert_eq!(a.clone() * b.clone(), AnyValue::Int32(50));
        assert_eq!(a.clone() / b.clone(), AnyValue::Int32(2));
    }

    #[test]
    fn test_numeric_operations_with_floats() {
        let a = AnyValue::Float64(10.0);
        let b = AnyValue::Float64(5.0);

        assert_eq!(a.clone() + b.clone(), AnyValue::Float64(15.0));
        assert_eq!(a.clone() - b.clone(), AnyValue::Float64(5.0));
        assert_eq!(a.clone() * b.clone(), AnyValue::Float64(50.0));
        assert_eq!(a.clone() / b.clone(), AnyValue::Float64(2.0));
    }
}
