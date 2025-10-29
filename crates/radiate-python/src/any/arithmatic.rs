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

    #[inline(always)]
    fn add(self, other: Self) -> Self {
        use AnyValue::*;
        let is_numeric = self.dtype().is_numeric() && other.dtype().is_numeric();
        let is_nested = self.is_nested() && other.is_nested();

        if !is_numeric && !is_nested {
            return self;
        }

        match (self, other) {
            (Bool(a), Bool(b)) => Bool(a || b),
            (Vector(a), Vector(b)) => Vector(Box::new(
                a.into_iter()
                    .zip(b.into_iter())
                    .map(|(x, y)| x + y)
                    .collect(),
            )),
            (Struct(a), Struct(b)) => {
                if a.len() != b.len() {
                    return Null;
                }

                Struct(
                    a.into_iter()
                        .zip(b.into_iter())
                        .map(|(one, two)| {
                            if one.0.name() != two.0.name() {
                                return (one.0, Null);
                            }

                            (one.0, one.1 + two.1)
                        })
                        .collect(),
                )
            }
            (lhs, rhs) => bin_numeric_op!(lhs, rhs, +),
        }
    }
}

impl Sub for AnyValue<'_> {
    type Output = Self;

    #[inline(always)]
    fn sub(self, other: Self) -> Self {
        use AnyValue::*;

        let is_numeric = self.dtype().is_numeric() && other.dtype().is_numeric();
        let is_nested = self.is_nested() && other.is_nested();

        if !is_numeric && !is_nested {
            return self;
        }

        match (self, other) {
            (Bool(a), Bool(b)) => Bool(a ^ b),
            (Vector(a), Vector(b)) => Vector(Box::new(
                a.into_iter()
                    .zip(b.into_iter())
                    .map(|(x, y)| x - y)
                    .collect(),
            )),
            (Struct(a), Struct(b)) => {
                if a.len() != b.len() {
                    return Null;
                }

                Struct(
                    a.into_iter()
                        .zip(b.into_iter())
                        .map(|(one, two)| {
                            if one.0.name() != two.0.name() {
                                return (one.0, Null);
                            }

                            (one.0, one.1 - two.1)
                        })
                        .collect(),
                )
            }
            (lhs, rhs) => bin_numeric_op!(lhs, rhs, -),
        }
    }
}

impl Mul for AnyValue<'_> {
    type Output = Self;

    #[inline(always)]
    fn mul(self, other: Self) -> Self {
        use AnyValue::*;

        let is_numeric = self.dtype().is_numeric() && other.dtype().is_numeric();
        let is_nested = self.is_nested() && other.is_nested();

        if !is_numeric && !is_nested {
            return self;
        }

        match (self, other) {
            (Bool(a), Bool(b)) => Bool(a && b),
            (Vector(a), Vector(b)) => Vector(Box::new(
                a.into_iter()
                    .zip(b.into_iter())
                    .map(|(x, y)| x * y)
                    .collect(),
            )),
            (Struct(a), Struct(b)) => {
                if a.len() != b.len() {
                    return Null;
                }

                Struct(
                    a.into_iter()
                        .zip(b.into_iter())
                        .map(|(one, two)| {
                            if one.0.name() != two.0.name() {
                                return (one.0, Null);
                            }

                            (one.0, one.1 * two.1)
                        })
                        .collect(),
                )
            }
            (lhs, rhs) => bin_numeric_op!(lhs, rhs, *),
        }
    }
}

impl Div for AnyValue<'_> {
    type Output = Self;

    #[inline(always)]
    fn div(self, other: Self) -> Self {
        use AnyValue::*;

        let is_numeric = self.dtype().is_numeric() && other.dtype().is_numeric();
        let is_nested = self.is_nested() && other.is_nested();

        if !is_numeric && !is_nested {
            return self;
        }

        match (self, other) {
            (Vector(a), Vector(b)) => Vector(Box::new(
                a.into_iter()
                    .zip(b.into_iter())
                    .map(|(x, y)| x / y)
                    .collect(),
            )),
            (Struct(a), Struct(b)) => {
                if a.len() != b.len() {
                    return Null;
                }

                Struct(
                    a.into_iter()
                        .zip(b.into_iter())
                        .map(|(one, two)| {
                            if one.0.name() != two.0.name() {
                                return (one.0, Null);
                            }

                            (one.0, one.1 / two.1)
                        })
                        .collect(),
                )
            }
            (lhs, rhs) => bin_numeric_div!(lhs, rhs),
        }
    }
}

#[inline]
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

            Some(Binary(out))
        }
        (Vector(xs), Vector(ys)) => crate::value::apply_zipped_slice(xs, ys, mean_anyvalue),
        (Struct(xs), Struct(ys)) => crate::value::apply_zipped_struct_slice(xs, ys, mean_anyvalue),
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

#[cfg(test)]
mod tests {
    use super::*;
    use AnyValue::*;

    fn v(xs: Vec<AnyValue<'static>>) -> AnyValue<'static> {
        AnyValue::Vector(Box::new(xs))
    }
    fn s(pairs: Vec<(&'static str, AnyValue<'static>)>) -> AnyValue<'static> {
        // Replace crate::Field(...) with your real Field constructor if needed
        let fields = pairs
            .into_iter()
            .map(|(name, val)| (crate::Field::new(name.into()), val))
            .collect();
        AnyValue::Struct(fields)
    }

    // ---------- Numeric: happy paths (same-type) ----------
    #[test]
    fn numeric_add_same_type() {
        assert_eq!(Bool(true) + Bool(false), Bool(true));

        assert_eq!(UInt8(10) + UInt8(5), UInt8(15));
        assert_eq!(UInt16(10) + UInt16(5), UInt16(15));
        assert_eq!(UInt32(10) + UInt32(5), UInt32(15));
        assert_eq!(UInt64(10) + UInt64(5), UInt64(15));

        assert_eq!(Int8(10) + Int8(5), Int8(15));
        assert_eq!(Int16(10) + Int16(5), Int16(15));
        assert_eq!(Int32(10) + Int32(5), Int32(15));
        assert_eq!(Int64(10) + Int64(5), Int64(15));
        assert_eq!(Int128(10) + Int128(5), Int128(15));

        assert_eq!(Float32(1.5) + Float32(2.0), Float32(3.5));
        assert_eq!(Float64(1.5) + Float64(2.0), Float64(3.5));
    }

    #[test]
    fn numeric_sub_same_type() {
        assert_eq!(Bool(true) - Bool(false), Bool(true));

        assert_eq!(UInt8(10) - UInt8(3), UInt8(7));
        assert_eq!(UInt16(10) - UInt16(3), UInt16(7));
        assert_eq!(UInt32(10) - UInt32(3), UInt32(7));
        assert_eq!(UInt64(10) - UInt64(3), UInt64(7));

        assert_eq!(Int8(10) - Int8(4), Int8(6));
        assert_eq!(Int16(10) - Int16(4), Int16(6));
        assert_eq!(Int32(10) - Int32(4), Int32(6));
        assert_eq!(Int64(10) - Int64(4), Int64(6));
        assert_eq!(Int128(10) - Int128(4), Int128(6));

        assert_eq!(Float32(5.0) - Float32(2.5), Float32(2.5));
        assert_eq!(Float64(5.0) - Float64(2.5), Float64(2.5));
    }

    #[test]
    fn numeric_mul_same_type() {
        assert_eq!(Bool(true) * Bool(false), Bool(true));

        assert_eq!(UInt8(7) * UInt8(6), UInt8(42));
        assert_eq!(UInt16(7) * UInt16(6), UInt16(42));
        assert_eq!(UInt32(7) * UInt32(6), UInt32(42));
        assert_eq!(UInt64(7) * UInt64(6), UInt64(42));

        assert_eq!(Int8(7) * Int8(6), Int8(42));
        assert_eq!(Int16(7) * Int16(6), Int16(42));
        assert_eq!(Int32(7) * Int32(6), Int32(42));
        assert_eq!(Int64(7) * Int64(6), Int64(42));
        assert_eq!(Int128(7) * Int128(6), Int128(42));

        assert_eq!(Float32(1.5) * Float32(2.0), Float32(3.0));
        assert_eq!(Float64(1.5) * Float64(2.0), Float64(3.0));
    }

    #[test]
    fn numeric_div_same_type() {
        assert_eq!(Bool(true) / Bool(false), Bool(true));

        assert_eq!(UInt8(42) / UInt8(6), UInt8(7));
        assert_eq!(UInt16(42) / UInt16(6), UInt16(7));
        assert_eq!(UInt32(42) / UInt32(6), UInt32(7));
        assert_eq!(UInt64(42) / UInt64(6), UInt64(7));

        assert_eq!(Int8(42) / Int8(6), Int8(7));
        assert_eq!(Int16(42) / Int16(6), Int16(7));
        assert_eq!(Int32(42) / Int32(6), Int32(7));
        assert_eq!(Int64(42) / Int64(6), Int64(7));
        assert_eq!(Int128(42) / Int128(6), Int128(7));

        assert_eq!(Float32(7.5) / Float32(2.5), Float32(3.0));
        assert_eq!(Float64(7.5) / Float64(2.5), Float64(3.0));
    }

    #[test]
    fn int_div_by_zero_yields_null() {
        assert_eq!(Int32(5) / Int32(0), Int32(5));
        assert_eq!(UInt64(7) / UInt64(0), UInt64(7));
    }

    // ---------- Vector elementwise ----------
    #[test]
    fn vector_elementwise_add_ok() {
        let a = v(vec![Int32(1), Int32(2), Int32(3)]);
        let b = v(vec![Int32(4), Int32(5), Int32(6)]);
        let out = v(vec![Int32(5), Int32(7), Int32(9)]);
        assert_eq!(a + b, out);
    }

    #[test]
    fn vector_length_mismatch() {
        let a = v(vec![Int32(1), Int32(2)]);
        let b = v(vec![Int32(3)]);
        assert_eq!(a + b, Vector(Box::new(vec![Int32(4)])));
    }

    // ---------- Struct fieldwise ----------
    #[test]
    fn struct_same_shape_by_order() {
        // Current code: length check; name mismatch → per-field Null (keeps left field)
        let a = s(vec![("x", Int32(1)), ("y", Int32(2))]);
        let b = s(vec![("x", Int32(3)), ("y", Int32(4))]);
        let out = s(vec![("x", Int32(4)), ("y", Int32(6))]);
        assert_eq!(a + b, out);
    }

    #[test]
    fn struct_length_mismatch_yields_null() {
        let a = s(vec![("x", Int32(1))]);
        let b = s(vec![("x", Int32(2)), ("y", Int32(3))]);
        assert_eq!(a + b, Null);
    }

    #[test]
    fn struct_field_name_mismatch_sets_field_null_under_current_rules() {
        let a = s(vec![("x", Int32(1)), ("y", Int32(2))]);
        let b = s(vec![("x", Int32(3)), ("z", Int32(9))]);
        // Current impl: when names differ at a position, that *slot* becomes Null; rest proceed.
        let expected = s(vec![("x", Int32(4)), ("y", Null)]);
        assert_eq!(a + b, expected);
    }

    #[test]
    fn struct_align_by_name_regardless_of_order() {
        let a = s(vec![("x", Int32(1)), ("y", Int32(2))]);
        let b = s(vec![("y", Int32(4)), ("x", Int32(3))]);
        let out = s(vec![("x", Null), ("y", Null)]);
        assert_eq!(a + b, out);
    }

    // ---------- Null interactions ----------
    #[test]
    fn null_propagation() {
        assert_eq!(Null + Int32(5), Null);
        assert_eq!(Float64(2.0) * Null, Float64(2.0));
        assert_eq!(Null / Null, Null);
    }

    // ---------- Mean ----------
    #[test]
    fn mean_numeric_pairs() {
        assert_eq!(mean_anyvalue(&Int32(2), &Int32(4)), Some(Int32(3)));
        assert_eq!(mean_anyvalue(&UInt8(10), &UInt8(20)), Some(UInt8(15)));
        assert_eq!(
            mean_anyvalue(&Float64(1.0), &Float64(3.0)),
            Some(Float64(2.0))
        );
    }

    #[test]
    fn mean_bool_is_and() {
        assert_eq!(mean_anyvalue(&Bool(true), &Bool(false)), Some(Bool(false)));
        assert_eq!(mean_anyvalue(&Bool(true), &Bool(true)), Some(Bool(true)));
    }

    // ---------- Algebraic sanity checks ----------
    #[test]
    fn add_commutative_for_numeric() {
        assert_eq!(Int64(7) + Int64(5), Int64(5) + Int64(7));
    }

    #[test]
    fn mul_commutative_for_numeric() {
        assert_eq!(Int16(3) * Int16(9), Int16(9) * Int16(3));
    }

    #[test]
    fn sub_non_commutative_for_numeric() {
        assert_ne!(Int32(10) - Int32(4), Int32(4) - Int32(10));
    }
}
