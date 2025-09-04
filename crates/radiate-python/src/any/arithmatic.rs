use crate::AnyValue;
use std::ops::{Add, Div, Mul, Sub};

/// Internal helper: perform `lhs <op> rhs` for all numeric AnyValue variants.
/// On type mismatch, returns `AnyValue::Null`.
macro_rules! bin_numeric_op {
    ($lhs:expr, $rhs:expr, $op:tt) => {{
        use AnyValue::*;
        match ($lhs, $rhs) {
            (Int8(a),    Int8(b))    => Int8(a $op b),
            (Int16(a),   Int16(b))   => Int16(a $op b),
            (Int32(a),   Int32(b))   => Int32(a $op b),
            (Int64(a),   Int64(b))   => Int64(a $op b),
            (Int128(a),  Int128(b))  => Int128(a $op b),
            (UInt8(a),   UInt8(b))   => UInt8(a $op b),
            (UInt16(a),  UInt16(b))  => UInt16(a $op b),
            (UInt32(a),  UInt32(b))  => UInt32(a $op b),
            (UInt64(a),  UInt64(b))  => UInt64(a $op b),
            (Float32(a), Float32(b)) => Float32(a $op b),
            (Float64(a), Float64(b)) => Float64(a $op b),
            _ => Null,
        }
    }};
}

/// Like `bin_numeric_op!`, but with integer safe divide (avoid div-by-zero).
macro_rules! bin_numeric_div {
    ($lhs:expr, $rhs:expr) => {{
        use AnyValue::*;
        match ($lhs, $rhs) {
            (Int8(a), Int8(b)) => Int8(if b == 0 { a } else { a / b }),
            (Int16(a), Int16(b)) => Int16(if b == 0 { a } else { a / b }),
            (Int32(a), Int32(b)) => Int32(if b == 0 { a } else { a / b }),
            (Int64(a), Int64(b)) => Int64(if b == 0 { a } else { a / b }),
            (Int128(a), Int128(b)) => Int128(if b == 0 { a } else { a / b }),
            (UInt8(a), UInt8(b)) => UInt8(if b == 0 { a } else { a / b }),
            (UInt16(a), UInt16(b)) => UInt16(if b == 0 { a } else { a / b }),
            (UInt32(a), UInt32(b)) => UInt32(if b == 0 { a } else { a / b }),
            (UInt64(a), UInt64(b)) => UInt64(if b == 0 { a } else { a / b }),
            (Float32(a), Float32(b)) => Float32(a / b), // IEEE handles inf/NaN
            (Float64(a), Float64(b)) => Float64(a / b),
            _ => Null,
        }
    }};
}

impl Add for AnyValue<'_> {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        let is_numeric = self.dtype().is_numeric() && other.dtype().is_numeric();
        let is_nested = self.dtype().is_nested() && other.dtype().is_nested();

        if !is_numeric && !is_nested {
            return self;
        }

        match (self, other) {
            (AnyValue::Bool(a), AnyValue::Bool(b)) => AnyValue::Bool(a && b),
            (AnyValue::Vector(a), AnyValue::Vector(b)) => AnyValue::Vector(Box::new(
                a.iter()
                    .cloned()
                    .zip(b.iter().cloned())
                    .map(|(x, y)| x + y)
                    .collect(),
            )),
            (AnyValue::Struct(a), AnyValue::Struct(b)) => AnyValue::Struct(
                a.iter()
                    .zip(b.iter())
                    .map(|((v1, f1), (v2, f2))| {
                        assert_eq!(f1.name(), f2.name());
                        (v1.clone() + v2.clone(), f1.clone())
                    })
                    .collect(),
            ),
            (lhs, rhs) => bin_numeric_op!(lhs, rhs, +),
        }
    }
}

impl Sub for AnyValue<'_> {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        let is_numeric = self.dtype().is_numeric() && other.dtype().is_numeric();
        let is_nested = self.dtype().is_nested() && other.dtype().is_nested();

        if !is_numeric && !is_nested {
            return self;
        }

        match (self, other) {
            (AnyValue::Vector(a), AnyValue::Vector(b)) => AnyValue::Vector(Box::new(
                a.iter()
                    .cloned()
                    .zip(b.iter().cloned())
                    .map(|(x, y)| x - y)
                    .collect(),
            )),
            (AnyValue::Struct(a), AnyValue::Struct(b)) => AnyValue::Struct(
                a.iter()
                    .zip(b.iter())
                    .map(|((v1, f1), (v2, f2))| {
                        assert_eq!(f1.name(), f2.name());
                        (v1.clone() - v2.clone(), f1.clone())
                    })
                    .collect(),
            ),
            (lhs, rhs) => bin_numeric_op!(lhs, rhs, -),
        }
    }
}

impl Mul for AnyValue<'_> {
    type Output = Self;
    fn mul(self, other: Self) -> Self {
        let is_numeric = self.dtype().is_numeric() && other.dtype().is_numeric();
        let is_nested = self.dtype().is_nested() && other.dtype().is_nested();

        if !is_numeric && !is_nested {
            return self;
        }

        match (self, other) {
            (AnyValue::Vector(a), AnyValue::Vector(b)) => AnyValue::Vector(Box::new(
                a.iter()
                    .cloned()
                    .zip(b.iter().cloned())
                    .map(|(x, y)| x * y)
                    .collect(),
            )),
            (AnyValue::Struct(a), AnyValue::Struct(b)) => AnyValue::Struct(
                a.iter()
                    .zip(b.iter())
                    .map(|((v1, f1), (v2, f2))| {
                        assert_eq!(f1.name(), f2.name());
                        (v1.clone() * v2.clone(), f1.clone())
                    })
                    .collect(),
            ),
            (lhs, rhs) => bin_numeric_op!(lhs, rhs, *),
        }
    }
}

impl Div for AnyValue<'_> {
    type Output = Self;
    fn div(self, other: Self) -> Self {
        let is_numeric = self.dtype().is_numeric() && other.dtype().is_numeric();
        let is_nested = self.dtype().is_nested() && other.dtype().is_nested();

        if !is_numeric && !is_nested {
            return self;
        }

        match (self, other) {
            (AnyValue::Vector(a), AnyValue::Vector(b)) => AnyValue::Vector(Box::new(
                a.iter()
                    .cloned()
                    .zip(b.iter().cloned())
                    .map(|(x, y)| x / y)
                    .collect(),
            )),
            (AnyValue::Struct(a), AnyValue::Struct(b)) => AnyValue::Struct(
                a.iter()
                    .zip(b.iter())
                    .map(|((v1, f1), (v2, f2))| {
                        assert_eq!(f1.name(), f2.name());
                        (v1.clone() / v2.clone(), f1.clone())
                    })
                    .collect(),
            ),
            (lhs, rhs) => bin_numeric_div!(lhs, rhs),
        }
    }
}

pub fn mean_anyvalue(one: &AnyValue<'_>, two: &AnyValue<'_>) -> Option<AnyValue<'static>> {
    use AnyValue::*;
    if let Some(v) = mean_numeric(one, two) {
        return Some(v);
    }

    match (one, two) {
        (Bool(x), Bool(y)) => Some(Bool(*x && *y)),
        (Binary(x), Binary(y)) => {
            let m = x.len().min(y.len());
            let mut out = Vec::with_capacity(m);

            for i in 0..m {
                out.push(((x[i] as u16 + y[i] as u16) / 2) as u8);
            }

            Some(AnyValue::Binary(out))
        }
        (Vector(xs), Vector(ys)) => crate::value::zip_slice_any_value_apply(xs, ys, mean_anyvalue),
        (Struct(xs), Struct(ys)) => crate::value::zip_struct_any_value_apply(xs, ys, mean_anyvalue),
        _ => None,
    }
}

#[inline]
fn mean_numeric(a: &AnyValue<'_>, b: &AnyValue<'_>) -> Option<AnyValue<'static>> {
    use AnyValue::*;
    let out = match (a, b) {
        (UInt8(x), UInt8(y)) => UInt8(((u16::from(*x) + u16::from(*y)) / 2) as u8),
        (UInt16(x), UInt16(y)) => UInt16(((u32::from(*x) + u32::from(*y)) / 2) as u16),
        (UInt32(x), UInt32(y)) => UInt32(((u64::from(*x) + u64::from(*y)) / 2) as u32),
        (UInt64(x), UInt64(y)) => UInt64(((u128::from(*x) + u128::from(*y)) / 2) as u64),

        (Int8(x), Int8(y)) => Int8(*x + ((*y as i16 - *x as i16) / 2) as i8),
        (Int16(x), Int16(y)) => Int16(*x + ((*y as i32 - *x as i32) / 2) as i16),
        (Int32(x), Int32(y)) => Int32(*x + ((*y as i64 - *x as i64) / 2) as i32),
        (Int64(x), Int64(y)) => {
            let dx = (*y as i128) - (*x as i128);
            Int64(*x + (dx / 2) as i64)
        }
        (Int128(x), Int128(y)) => Int128(*x + ((*y - *x) / 2)),

        (Float32(x), Float32(y)) => Float32((*x + *y) / 2.0),
        (Float64(x), Float64(y)) => Float64((*x + *y) / 2.0),

        _ => return None,
    };

    Some(out)
}
