use radiate_error::radiate_bail;

use crate::AnyValue;
use std::ops::{Add, BitAnd, BitOr, Div, Mul, Not, Rem, Sub};
use std::time::Duration;

// Arithmetic on `AnyValue` follows one set of rules:
//
// - Numeric operands are promoted to a common type, then a single checked
//   operation runs:
//     - either side a float  → that float width (the wider one if both are)
//     - both integers        → the wider width, signed if either side is
//                              signed (`Usize` is treated as `u64`)
//   Integer overflow, and division or remainder by zero (int or float),
//   produce `Null`.
// - Durations: `Duration ± Duration → Duration`, `Duration × number`,
//   `number × Duration` and `Duration ÷ number → Duration`, and
//   `Duration ÷ Duration → Float32` (a ratio). Negative or overflowing
//   results produce `Null`.
// - `Vector`/`Vector` and `Dict`/`Dict` apply elementwise.
// - Every other combination (Null, Bool, strings, nested with scalar) → `Null`.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Op {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NumKind {
    Int { signed: bool, bits: u8 },
    Float { bits: u8 },
}

/// A numeric operand widened into a carrier every source type fits in.
#[derive(Clone, Copy, Debug)]
enum Num {
    I(i128),
    U(u128),
    F(f64),
}

impl Num {
    #[inline]
    fn as_f64(self) -> f64 {
        match self {
            Num::I(x) => x as f64,
            Num::U(x) => x as f64,
            Num::F(x) => x,
        }
    }

    #[inline]
    fn as_i128(self) -> Option<i128> {
        match self {
            Num::I(x) => Some(x),
            Num::U(x) => i128::try_from(x).ok(),
            Num::F(_) => None,
        }
    }

    #[inline]
    fn as_u128(self) -> Option<u128> {
        match self {
            Num::U(x) => Some(x),
            Num::I(x) => u128::try_from(x).ok(),
            Num::F(_) => None,
        }
    }
}

#[inline]
fn classify(value: &AnyValue<'_>) -> Option<(NumKind, Num)> {
    use AnyValue::*;
    let uint = |bits| NumKind::Int {
        signed: false,
        bits,
    };
    let int = |bits| NumKind::Int { signed: true, bits };

    Some(match value {
        UInt8(x) => (uint(8), Num::U(u128::from(*x))),
        UInt16(x) => (uint(16), Num::U(u128::from(*x))),
        UInt32(x) => (uint(32), Num::U(u128::from(*x))),
        UInt64(x) => (uint(64), Num::U(u128::from(*x))),
        UInt128(x) => (uint(128), Num::U(*x)),
        Usize(x) => (uint(64), Num::U(*x as u128)),

        Int8(x) => (int(8), Num::I(i128::from(*x))),
        Int16(x) => (int(16), Num::I(i128::from(*x))),
        Int32(x) => (int(32), Num::I(i128::from(*x))),
        Int64(x) => (int(64), Num::I(i128::from(*x))),
        Int128(x) => (int(128), Num::I(*x)),

        Float32(x) => (NumKind::Float { bits: 32 }, Num::F(f64::from(*x))),
        Float64(x) => (NumKind::Float { bits: 64 }, Num::F(*x)),
        _ => return None,
    })
}

#[inline]
fn promote(a: NumKind, b: NumKind) -> NumKind {
    use NumKind::*;
    match (a, b) {
        (Float { bits: x }, Float { bits: y }) => Float { bits: x.max(y) },
        (Float { .. }, Int { .. }) => a,
        (Int { .. }, Float { .. }) => b,
        (
            Int {
                signed: s1,
                bits: b1,
            },
            Int {
                signed: s2,
                bits: b2,
            },
        ) => Int {
            signed: s1 || s2,
            bits: b1.max(b2),
        },
    }
}

#[inline]
fn narrow_unsigned(value: u128, bits: u8) -> Option<AnyValue<'static>> {
    use AnyValue::*;
    Some(match bits {
        8 => UInt8(u8::try_from(value).ok()?),
        16 => UInt16(u16::try_from(value).ok()?),
        32 => UInt32(u32::try_from(value).ok()?),
        64 => UInt64(u64::try_from(value).ok()?),
        _ => UInt128(value),
    })
}

#[inline]
fn narrow_signed(value: i128, bits: u8) -> Option<AnyValue<'static>> {
    use AnyValue::*;
    Some(match bits {
        8 => Int8(i8::try_from(value).ok()?),
        16 => Int16(i16::try_from(value).ok()?),
        32 => Int32(i32::try_from(value).ok()?),
        64 => Int64(i64::try_from(value).ok()?),
        _ => Int128(value),
    })
}

#[inline]
fn narrow_float(value: f64, bits: u8) -> AnyValue<'static> {
    if bits == 32 {
        AnyValue::Float32(value as f32)
    } else {
        AnyValue::Float64(value)
    }
}

#[inline]
fn numeric_op(lhs: &AnyValue<'_>, rhs: &AnyValue<'_>, op: Op) -> Option<AnyValue<'static>> {
    let (lhs_kind, a) = classify(lhs)?;
    let (rhs_kind, b) = classify(rhs)?;

    match promote(lhs_kind, rhs_kind) {
        NumKind::Float { bits } => {
            let (x, y) = (a.as_f64(), b.as_f64());
            if matches!(op, Op::Div | Op::Rem) && y == 0.0 {
                return None;
            }

            // For f32 operands, computing in f64 and rounding once is exact
            // for + - * / % (f64 has more than 2×24+2 mantissa bits).
            let result = match op {
                Op::Add => x + y,
                Op::Sub => x - y,
                Op::Mul => x * y,
                Op::Div => x / y,
                Op::Rem => x % y,
            };

            Some(narrow_float(result, bits))
        }
        NumKind::Int {
            signed: false,
            bits,
        } => {
            let (x, y) = (a.as_u128()?, b.as_u128()?);
            let result = match op {
                Op::Add => x.checked_add(y),
                Op::Sub => x.checked_sub(y),
                Op::Mul => x.checked_mul(y),
                Op::Div => x.checked_div(y),
                Op::Rem => x.checked_rem(y),
            }?;

            narrow_unsigned(result, bits)
        }
        NumKind::Int { signed: true, bits } => {
            let (x, y) = (a.as_i128()?, b.as_i128()?);
            let result = match op {
                Op::Add => x.checked_add(y),
                Op::Sub => x.checked_sub(y),
                Op::Mul => x.checked_mul(y),
                Op::Div => x.checked_div(y),
                Op::Rem => x.checked_rem(y),
            }?;

            narrow_signed(result, bits)
        }
    }
}

#[inline]
fn scale_duration(duration: &Duration, factor: f64) -> Option<AnyValue<'static>> {
    if !factor.is_finite() || factor < 0.0 {
        return None;
    }

    Duration::try_from_secs_f64(duration.as_secs_f64() * factor)
        .ok()
        .map(AnyValue::Duration)
}

#[inline]
fn duration_op(lhs: &AnyValue<'_>, rhs: &AnyValue<'_>, op: Op) -> Option<AnyValue<'static>> {
    use AnyValue::Duration as Dur;

    match (lhs, rhs, op) {
        (Dur(a), Dur(b), Op::Add) => a.checked_add(*b).map(Dur),
        (Dur(a), Dur(b), Op::Sub) => a.checked_sub(*b).map(Dur),
        (Dur(a), Dur(b), Op::Div) => {
            if b.is_zero() {
                None
            } else {
                Some(AnyValue::Float32(
                    (a.as_secs_f64() / b.as_secs_f64()) as f32,
                ))
            }
        }
        (Dur(d), n, Op::Mul) if classify(n).is_some() => scale_duration(d, n.extract::<f64>()?),
        (n, Dur(d), Op::Mul) if classify(n).is_some() => scale_duration(d, n.extract::<f64>()?),
        (Dur(d), n, Op::Div) if classify(n).is_some() => {
            let divisor = n.extract::<f64>()?;
            if divisor == 0.0 {
                None
            } else {
                scale_duration(d, 1.0 / divisor)
            }
        }
        _ => None,
    }
}

fn arith<'a>(lhs: AnyValue<'a>, rhs: AnyValue<'a>, op: Op) -> AnyValue<'a> {
    use AnyValue::*;

    match (lhs, rhs) {
        (Vector(a), Vector(b)) => {
            Vector(a.into_iter().zip(b).map(|(x, y)| arith(x, y, op)).collect())
        }
        (Dict(a), Dict(b)) => {
            if a.len() != b.len() {
                return Null;
            }

            Dict(
                a.into_iter()
                    .zip(b)
                    .map(|(one, two)| {
                        if one.0 != two.0 {
                            return (one.0, one.1, Null);
                        }

                        (one.0, one.1, arith(one.2, two.2, op))
                    })
                    .collect(),
            )
        }
        (l, r) => numeric_op(&l, &r, op)
            .or_else(|| duration_op(&l, &r, op))
            .unwrap_or(Null),
    }
}

impl Add for AnyValue<'_> {
    type Output = Self;

    #[inline]
    fn add(self, other: Self) -> Self {
        arith(self, other, Op::Add)
    }
}

impl Sub for AnyValue<'_> {
    type Output = Self;

    #[inline]
    fn sub(self, other: Self) -> Self {
        arith(self, other, Op::Sub)
    }
}

impl Mul for AnyValue<'_> {
    type Output = Self;

    #[inline]
    fn mul(self, other: Self) -> Self {
        arith(self, other, Op::Mul)
    }
}

impl Div for AnyValue<'_> {
    type Output = Self;

    #[inline]
    fn div(self, other: Self) -> Self {
        arith(self, other, Op::Div)
    }
}

impl Rem for AnyValue<'_> {
    type Output = Self;

    #[inline]
    fn rem(self, other: Self) -> Self {
        arith(self, other, Op::Rem)
    }
}

impl<'a> BitAnd for AnyValue<'a> {
    type Output = AnyValue<'static>;

    fn bitand(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (AnyValue::Bool(a), AnyValue::Bool(b)) => AnyValue::Bool(a & b),
            _ => AnyValue::Null,
        }
    }
}

impl<'a> BitOr for AnyValue<'a> {
    type Output = AnyValue<'static>;

    fn bitor(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (AnyValue::Bool(a), AnyValue::Bool(b)) => AnyValue::Bool(a | b),
            _ => AnyValue::Null,
        }
    }
}

impl<'a> Not for AnyValue<'a> {
    type Output = AnyValue<'static>;

    fn not(self) -> Self::Output {
        match self {
            AnyValue::Bool(v) => AnyValue::Bool(!v),
            _ => AnyValue::Null,
        }
    }
}

/// `base ^ exp` under the same promotion rules as the arithmetic operators.
///
/// Non-numeric operands are an error. Integer overflow, or a negative /
/// out-of-range integer exponent, produce `Null`.
#[inline]
pub fn pow_anyvalue(
    base: &AnyValue<'_>,
    exp: &AnyValue<'_>,
) -> Result<AnyValue<'static>, radiate_error::RadiateError> {
    let (Some((base_kind, b)), Some((exp_kind, e))) = (classify(base), classify(exp)) else {
        radiate_bail!(Expr: "Exponentiation is only supported for numeric types, got base {:?} and exponent {:?}", base, exp)
    };

    let result = match promote(base_kind, exp_kind) {
        NumKind::Float { bits } => Some(narrow_float(b.as_f64().powf(e.as_f64()), bits)),
        NumKind::Int { signed, bits } => {
            e.as_u128()
                .and_then(|e| u32::try_from(e).ok())
                .and_then(|e| {
                    if signed {
                        b.as_i128()?
                            .checked_pow(e)
                            .and_then(|r| narrow_signed(r, bits))
                    } else {
                        b.as_u128()?
                            .checked_pow(e)
                            .and_then(|r| narrow_unsigned(r, bits))
                    }
                })
        }
    };

    Ok(result.unwrap_or(AnyValue::Null))
}

#[inline]
#[allow(dead_code)]
fn mean_anyvalue(one: &AnyValue<'_>, two: &AnyValue<'_>) -> Option<AnyValue<'static>> {
    use AnyValue::*;
    if let Some(v) = mean_numeric(one, two) {
        return Some(v);
    }

    match (one, two) {
        (Bool(x), Bool(y)) => Some(Bool(*x && *y)),

        (Vector(xs), Vector(ys)) => super::value::apply_zipped_slice(xs, ys, mean_anyvalue),
        (Dict(xs), Dict(ys)) => super::value::apply_zipped_struct_slice(xs, ys, mean_anyvalue),
        _ => None,
    }
}

#[inline]
#[allow(dead_code)]
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
    use crate::SmallStr;

    use super::*;
    use AnyValue::*;

    fn make_vec(xs: Vec<AnyValue<'static>>) -> AnyValue<'static> {
        AnyValue::Vector(xs)
    }

    fn make_dict(pairs: Vec<(&'static str, AnyValue<'static>)>) -> AnyValue<'static> {
        let fields = pairs
            .into_iter()
            .map(|(name, val)| (SmallStr::from(name), val.dtype(), val))
            .collect();
        AnyValue::Dict(fields)
    }

    // ---------- Numeric: happy paths (same-type) ----------
    #[test]
    fn numeric_add_same_type() {
        assert_eq!(Bool(true) + Bool(false), Null);

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
        assert_eq!(Bool(true) - Bool(false), Null);

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
        assert_eq!(Bool(true) * Bool(false), Null);

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
        assert_eq!(Bool(true) / Bool(false), Null);

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
        assert_eq!(Int32(5) / Int32(0), Null);
        assert_eq!(UInt64(7) / UInt64(0), Null);
    }

    // ---------- Vector elementwise ----------
    #[test]
    fn vector_elementwise_add_ok() {
        let a = make_vec(vec![Int32(1), Int32(2), Int32(3)]);
        let b = make_vec(vec![Int32(4), Int32(5), Int32(6)]);
        let out = make_vec(vec![Int32(5), Int32(7), Int32(9)]);
        assert_eq!(a + b, out);
    }

    #[test]
    fn vector_length_mismatch() {
        let a = make_vec(vec![Int32(1), Int32(2)]);
        let b = make_vec(vec![Int32(3)]);
        assert_eq!(a + b, Vector(vec![Int32(4)]));
    }

    // ---------- Struct fieldwise ----------
    #[test]
    fn struct_same_shape_by_order() {
        // Current code: length check; name mismatch → per-field Null (keeps left field)
        let a = make_dict(vec![("x", Int32(1)), ("y", Int32(2))]);
        let b = make_dict(vec![("x", Int32(3)), ("y", Int32(4))]);
        let out = make_dict(vec![("x", Int32(4)), ("y", Int32(6))]);
        assert_eq!(a + b, out);
    }

    #[test]
    fn struct_length_mismatch_yields_null() {
        let a = make_dict(vec![("x", Int32(1))]);
        let b = make_dict(vec![("x", Int32(2)), ("y", Int32(3))]);
        assert_eq!(a + b, Null);
    }

    #[test]
    fn struct_field_name_mismatch_sets_field_null_under_current_rules() {
        let a = make_dict(vec![("x", Int32(1)), ("y", Int32(2))]);
        let b = make_dict(vec![("x", Int32(3)), ("z", Int32(9))]);
        // Current impl: when names differ at a position, that *slot* becomes Null; rest proceed.
        let expected = make_dict(vec![("x", Int32(4)), ("y", Null)]);
        assert_eq!(a + b, expected);
    }

    #[test]
    fn struct_align_by_name_regardless_of_order() {
        let a = make_dict(vec![("x", Int32(1)), ("y", Int32(2))]);
        let b = make_dict(vec![("y", Int32(4)), ("x", Int32(3))]);
        let out = make_dict(vec![("x", Null), ("y", Null)]);
        assert_eq!(a + b, out);
    }

    // ---------- Null interactions ----------
    #[test]
    fn null_propagation() {
        assert_eq!(Null + Int32(5), Null);
        assert_eq!(Float64(2.0) * Null, Null);
        assert_eq!(Null / Null, Null);
    }

    // ---------- Mixed-type promotion ----------
    #[test]
    fn int_with_float_promotes_to_the_float() {
        assert_eq!(Float32(1.5) * Int32(2), Float32(3.0));
        assert_eq!(Int64(2) * Float32(1.5), Float32(3.0));
        assert_eq!(Float32(9.0) / Int32(3), Float32(3.0));
        assert_eq!(UInt64(3) + Float64(0.5), Float64(3.5));
        assert_eq!(Float32(1.0) + Float64(0.5), Float64(1.5));
    }

    #[test]
    fn mixed_int_widths_promote_to_wider() {
        assert_eq!(Int8(1) + Int64(2), Int64(3));
        assert_eq!(UInt8(1) + UInt32(2), UInt32(3));
        assert_eq!(Usize(4) + UInt8(1), UInt64(5));
    }

    #[test]
    fn signed_with_unsigned_promotes_to_signed() {
        assert_eq!(UInt64(3) - Int32(5), Int64(-2));
        assert_eq!(Int8(-1) + UInt32(1), Int32(0));
    }

    #[test]
    fn int_division_still_truncates() {
        assert_eq!(Int32(7) / Int32(2), Int32(3));
        assert_eq!(UInt64(7) / Int32(2), Int64(3));
    }

    #[test]
    fn mixed_division_never_panics() {
        assert_eq!(Float32(1.0) / Int32(0), Null);
        assert_eq!(Usize(5) / Float32(2.0), Float32(2.5));
        assert_eq!(UInt64(5) / Float32(0.0), Null);
    }

    // ---------- Overflow ----------
    #[test]
    fn integer_overflow_yields_null() {
        assert_eq!(UInt64(3) - UInt64(5), Null);
        assert_eq!(UInt8(255) + UInt8(1), Null);
        assert_eq!(Int8(-128) * Int8(-1), Null);
        assert_eq!(Int32(i32::MIN) / Int32(-1), Null);
        assert_eq!(Int32(5) % Int32(0), Null);
    }

    // ---------- Duration ----------
    #[test]
    fn duration_arithmetic() {
        use std::time::Duration as D;
        let s = |x: f64| Duration(D::from_secs_f64(x));

        assert_eq!(s(1.0) + s(0.5), s(1.5));
        assert_eq!(s(1.0) - s(0.25), s(0.75));
        assert_eq!(s(0.5) - s(1.0), Null);

        assert_eq!(s(1.5) * Int32(2), s(3.0));
        assert_eq!(Float32(2.0) * s(1.5), s(3.0));
        assert_eq!(s(3.0) / Int32(2), s(1.5));
        assert_eq!(s(3.0) / Float32(0.0), Null);
        assert_eq!(s(1.0) * Float32(-1.0), Null);

        assert_eq!(s(3.0) / s(1.5), Float32(2.0));
        assert_eq!(s(3.0) / s(0.0), Null);

        assert_eq!(s(1.0) + Int32(1), Null);
        assert_eq!(Int32(1) / s(1.0), Null);
    }

    // ---------- Non-numeric ----------
    #[test]
    fn non_numeric_operands_yield_null_regardless_of_side() {
        assert_eq!(Int32(5) + Null, Null);
        assert_eq!(Null + Int32(5), Null);
        assert_eq!(Str("a") + Int32(1), Null);
        assert_eq!(make_vec(vec![Int32(1)]) + Int32(1), Null);
    }

    // ---------- Pow ----------
    #[test]
    fn pow_promotes_and_checks() {
        assert_eq!(
            pow_anyvalue(&Float32(2.0), &Int32(3)).unwrap(),
            Float32(8.0)
        );
        assert_eq!(pow_anyvalue(&Int32(2), &Int8(10)).unwrap(), Int32(1024));
        assert_eq!(pow_anyvalue(&Int32(2), &Int32(-1)).unwrap(), Null);
        assert_eq!(pow_anyvalue(&UInt8(2), &UInt8(8)).unwrap(), Null);
        assert!(pow_anyvalue(&Str("a"), &Int32(2)).is_err());
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
