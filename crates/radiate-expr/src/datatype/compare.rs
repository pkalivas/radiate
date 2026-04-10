use crate::AnyValue;
use std::cmp::Ordering;
use std::ops::{BitAnd, BitOr, Not};

impl<'a> PartialOrd for AnyValue<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        use AnyValue::*;

        match (self, other) {
            (Null, _) | (_, Null) => None,

            (Bool(a), Bool(b)) => a.partial_cmp(b),

            (Char(a), Char(b)) => a.partial_cmp(b),

            (Str(a), Str(b)) => a.partial_cmp(b),
            (Str(a), StrOwned(b)) => a.partial_cmp(&b.as_str()),
            (StrOwned(a), Str(b)) => a.as_str().partial_cmp(*b),
            (StrOwned(a), StrOwned(b)) => a.partial_cmp(b),

            (Duration(a), Duration(b)) => a.partial_cmp(b),

            (Vector(a), Vector(b)) => a.partial_cmp(b),
            (Slice(a), Slice(b)) => a.partial_cmp(b),
            (Slice(a), Vector(b)) => a.partial_cmp(&b.as_slice()),
            (Vector(a), Slice(b)) => a.as_slice().partial_cmp(b),

            // numeric cross-variant comparison
            (a, b) if a.is_numeric() && b.is_numeric() => {
                let lhs = a.clone().extract::<f64>()?;
                let rhs = b.clone().extract::<f64>()?;
                lhs.partial_cmp(&rhs)
            }

            // You can decide later whether Struct should support lexicographic ordering
            (Struct(a), Struct(b)) => {
                if a.len() != b.len() {
                    return None;
                }

                for ((ka, va), (kb, vb)) in a.iter().zip(b.iter()) {
                    if ka != kb {
                        return None; // Different field names, can't compare
                    }

                    match va.partial_cmp(vb) {
                        Some(Ordering::Equal) => continue,
                        non_eq => return non_eq, // Return the first non-equal ordering
                    }
                }

                Some(Ordering::Equal) // All fields are equal
            }

            _ => None,
        }
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
