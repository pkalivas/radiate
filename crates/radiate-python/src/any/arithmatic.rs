use crate::{AnyValue, Field};

/// Returns None when the types are incompatible (policy: you can choose to fallback differently).
pub fn mean_anyvalue(a: &AnyValue<'_>, b: &AnyValue<'_>) -> Option<AnyValue<'static>> {
    use AnyValue::*;
    // 1) numerics
    if let Some(v) = mean_numeric(a, b) {
        return Some(v);
    }

    match (a, b) {
        // 2) booleans — deterministic consensus (AND)
        (Bool(x), Bool(y)) => Some(Bool(*x && *y)),

        // 3) bytes — per-byte arith mean
        (Binary(x), Binary(y)) => Some(mean_binary(x, y)),

        // 4) elementwise containers
        (Vec(xs), Vec(ys)) => zip_vec_mean(xs, ys),
        (Struct(xs), Struct(ys)) => zip_struct_mean(xs, ys),

        // 5) unsupported combos
        _ => None,
    }
}

#[inline]
fn mean_numeric(a: &AnyValue<'_>, b: &AnyValue<'_>) -> Option<AnyValue<'static>> {
    use AnyValue::*;
    let out = match (a, b) {
        // widen unsigned to avoid overflow
        (UInt8(x), UInt8(y)) => UInt8(((u16::from(*x) + u16::from(*y)) / 2) as u8),
        (UInt16(x), UInt16(y)) => UInt16(((u32::from(*x) + u32::from(*y)) / 2) as u16),
        (UInt32(x), UInt32(y)) => UInt32(((u64::from(*x) + u64::from(*y)) / 2) as u32),
        (UInt64(x), UInt64(y)) => UInt64(((u128::from(*x) + u128::from(*y)) / 2) as u64),

        // midpoint formula for signed (avoid overflow): x + (y - x)/2
        (Int8(x), Int8(y)) => Int8(*x + ((*y as i16 - *x as i16) / 2) as i8),
        (Int16(x), Int16(y)) => Int16(*x + ((*y as i32 - *x as i32) / 2) as i16),
        (Int32(x), Int32(y)) => Int32(*x + ((*y as i64 - *x as i64) / 2) as i32),
        (Int64(x), Int64(y)) => {
            let dx = (*y as i128) - (*x as i128);
            Int64(*x + (dx / 2) as i64)
        }
        (Int128(x), Int128(y)) => Int128(*x + ((*y - *x) / 2)),

        // floats are easy
        (Float32(x), Float32(y)) => Float32((*x + *y) / 2.0),
        (Float64(x), Float64(y)) => Float64((*x + *y) / 2.0),

        _ => return None,
    };
    Some(out)
}

#[inline]
fn mean_binary(a: &[u8], b: &[u8]) -> AnyValue<'static> {
    let m = a.len().min(b.len());
    let mut out = Vec::with_capacity(m);
    for i in 0..m {
        out.push(((a[i] as u16 + b[i] as u16) / 2) as u8);
    }
    AnyValue::Binary(out)
}

#[inline]
fn zip_vec_mean(a: &[AnyValue<'_>], b: &[AnyValue<'_>]) -> Option<AnyValue<'static>> {
    if a.len() != b.len() {
        return None;
    }
    let mut out = Vec::with_capacity(a.len());
    for (x, y) in a.iter().zip(b) {
        out.push(mean_anyvalue(x, y)?);
    }
    Some(AnyValue::Vec(Box::new(out)))
}

#[inline]
fn zip_struct_mean(
    a: &[(AnyValue<'_>, Field)],
    b: &[(AnyValue<'_>, Field)],
) -> Option<AnyValue<'static>> {
    if a.len() != b.len() {
        return None;
    }
    // require matching field names, same order
    if !a
        .iter()
        .map(|(_, f)| f.name())
        .eq(b.iter().map(|(_, f)| f.name()))
    {
        return None;
    }
    let mut out = Vec::with_capacity(a.len());
    for ((va, fa), (vb, fb)) in a.iter().zip(b.iter()) {
        debug_assert_eq!(fa.name(), fb.name());
        out.push((mean_anyvalue(va, vb)?, fa.clone()));
    }
    Some(AnyValue::Struct(out))
}
