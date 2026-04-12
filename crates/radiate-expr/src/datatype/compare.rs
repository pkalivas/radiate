use crate::AnyValue;
use std::cmp::Ordering;

impl<'a> AnyValue<'a> {
    #[inline]
    fn variant_rank(&self) -> u8 {
        match self {
            AnyValue::Null => 0,
            AnyValue::Bool(_) => 1,

            AnyValue::UInt8(_) => 2,
            AnyValue::UInt16(_) => 3,
            AnyValue::UInt32(_) => 4,
            AnyValue::UInt64(_) => 5,
            AnyValue::UInt128(_) => 6,

            AnyValue::Int8(_) => 7,
            AnyValue::Int16(_) => 8,
            AnyValue::Int32(_) => 9,
            AnyValue::Int64(_) => 10,
            AnyValue::Int128(_) => 11,

            AnyValue::Float32(_) => 12,
            AnyValue::Float64(_) => 13,

            AnyValue::Duration(_) => 14,

            AnyValue::Char(_) => 15,
            AnyValue::Str(_) => 16,
            AnyValue::StrOwned(_) => 17,

            AnyValue::Slice(_) => 18,
            AnyValue::Vector(_) => 19,

            AnyValue::Struct(_) => 20,
        }
    }

    fn cmp_same_variant(&self, other: &Self) -> Ordering {
        use AnyValue::*;

        match (self, other) {
            (Null, Null) => Ordering::Equal,
            (Bool(a), Bool(b)) => a.cmp(b),

            (UInt8(a), UInt8(b)) => a.cmp(b),
            (UInt16(a), UInt16(b)) => a.cmp(b),
            (UInt32(a), UInt32(b)) => a.cmp(b),
            (UInt64(a), UInt64(b)) => a.cmp(b),
            (UInt128(a), UInt128(b)) => a.cmp(b),

            (Int8(a), Int8(b)) => a.cmp(b),
            (Int16(a), Int16(b)) => a.cmp(b),
            (Int32(a), Int32(b)) => a.cmp(b),
            (Int64(a), Int64(b)) => a.cmp(b),
            (Int128(a), Int128(b)) => a.cmp(b),

            (Float32(a), Float32(b)) => a.total_cmp(b),
            (Float64(a), Float64(b)) => a.total_cmp(b),

            (Duration(a), Duration(b)) => a.cmp(b),

            (Char(a), Char(b)) => a.cmp(b),
            (Str(a), Str(b)) => a.cmp(b),
            (StrOwned(a), StrOwned(b)) => a.cmp(b),

            (Slice(a), Slice(b)) => a.iter().cmp(b.iter()),
            (Vector(a), Vector(b)) => a.iter().cmp(b.iter()),

            (Struct(a), Struct(b)) => {
                let mut i = 0;
                while i < a.len() && i < b.len() {
                    let (fa, va) = &a[i];
                    let (fb, vb) = &b[i];

                    match fa.name().cmp(fb.name()) {
                        Ordering::Equal => {}
                        non_eq => return non_eq,
                    }

                    match va.cmp(vb) {
                        Ordering::Equal => {}
                        non_eq => return non_eq,
                    }

                    i += 1;
                }

                a.len().cmp(&b.len())
            }
            _ => unreachable!("cmp_same_variant called with different variants"),
        }
    }

    fn fuzzy_cmp(&self, other: &Self) -> Option<Ordering> {
        use AnyValue::*;

        if self.is_float() && other.is_float() {
            return self.cmp_float(other);
        } else if self.is_int() && other.is_int() {
            return self.cmp_int(other);
        } else if self.is_string() && other.is_string() {
            return self.cmp_str(other);
        } else {
            let res = match (self, other) {
                (Null, Null) => Ordering::Equal,
                (Bool(a), Bool(b)) => a.cmp(b),

                (Vector(a), Vector(b)) => a.iter().cmp(b.iter()),
                (Slice(a), Slice(b)) => a.iter().cmp(b.iter()),

                _ => return None,
            };

            Some(res)
        }
    }

    fn cmp_float(&self, other: &Self) -> Option<Ordering> {
        use AnyValue::*;

        match (self, other) {
            (Float32(a), Float32(b)) => Some(a.total_cmp(b)),
            (Float64(a), Float64(b)) => Some(a.total_cmp(b)),
            (Float32(a), Float64(b)) => Some((*a as f64).total_cmp(b)),
            (Float64(a), Float32(b)) => Some(a.total_cmp(&(*b as f64))),
            _ => None,
        }
    }

    fn cmp_str(&self, other: &Self) -> Option<Ordering> {
        use AnyValue::*;

        match (self, other) {
            (Str(a), Str(b)) => Some(a.cmp(b)),
            (StrOwned(a), StrOwned(b)) => Some(a.cmp(b)),
            (Char(a), Char(b)) => Some(a.cmp(b)),

            (Str(a), StrOwned(b)) => Some(a.cmp(&b.as_str())),
            (StrOwned(a), Str(b)) => Some(a.as_str().cmp(b)),
            _ => None,
        }
    }

    fn cmp_int(&self, other: &Self) -> Option<Ordering> {
        use AnyValue::*;

        match (self, other) {
            (Int8(a), Int8(b)) => Some(a.cmp(b)),
            (Int16(a), Int16(b)) => Some(a.cmp(b)),
            (Int32(a), Int32(b)) => Some(a.cmp(b)),
            (Int64(a), Int64(b)) => Some(a.cmp(b)),
            (Int128(a), Int128(b)) => Some(a.cmp(b)),

            (UInt8(a), UInt8(b)) => Some(a.cmp(b)),
            (UInt16(a), UInt16(b)) => Some(a.cmp(b)),
            (UInt32(a), UInt32(b)) => Some(a.cmp(b)),
            (UInt64(a), UInt64(b)) => Some(a.cmp(b)),
            (UInt128(a), UInt128(b)) => Some(a.cmp(b)),

            (Int8(a), Int16(b)) => Some((*a as i16).cmp(b)),
            (Int8(a), Int32(b)) => Some((*a as i32).cmp(b)),
            (Int8(a), Int64(b)) => Some((*a as i64).cmp(b)),
            (Int8(a), Int128(b)) => Some((*a as i128).cmp(b)),

            (Int16(a), Int8(b)) => Some(a.cmp(&(*b as i16))),
            (Int16(a), Int32(b)) => Some((*a as i32).cmp(b)),
            (Int16(a), Int64(b)) => Some((*a as i64).cmp(b)),
            (Int16(a), Int128(b)) => Some((*a as i128).cmp(b)),

            (Int32(a), Int8(b)) => Some(a.cmp(&(*b as i32))),
            (Int32(a), Int16(b)) => Some(a.cmp(&(*b as i32))),
            (Int32(a), Int64(b)) => Some((*a as i64).cmp(b)),
            (Int32(a), Int128(b)) => Some((*a as i128).cmp(b)),

            (Int64(a), Int8(b)) => Some(a.cmp(&(*b as i64))),
            (Int64(a), Int16(b)) => Some(a.cmp(&(*b as i64))),
            (Int64(a), Int32(b)) => Some(a.cmp(&(*b as i64))),
            (Int64(a), Int128(b)) => Some((*a as i128).cmp(b)),

            (UInt8(a), UInt16(b)) => Some((*a as u16).cmp(b)),
            (UInt8(a), UInt32(b)) => Some((*a as u32).cmp(b)),
            (UInt8(a), UInt64(b)) => Some((*a as u64).cmp(b)),
            (UInt8(a), UInt128(b)) => Some((*a as u128).cmp(b)),

            (UInt16(a), UInt8(b)) => Some(a.cmp(&(*b as u16))),
            (UInt16(a), UInt32(b)) => Some((*a as u32).cmp(b)),
            (UInt16(a), UInt64(b)) => Some((*a as u64).cmp(b)),
            (UInt16(a), UInt128(b)) => Some((*a as u128).cmp(b)),

            (UInt32(a), UInt8(b)) => Some(a.cmp(&(*b as u32))),
            (UInt32(a), UInt16(b)) => Some(a.cmp(&(*b as u32))),
            (UInt32(a), UInt64(b)) => Some((*a as u64).cmp(b)),
            (UInt32(a), UInt128(b)) => Some((*a as u128).cmp(b)),

            (UInt64(a), UInt8(b)) => Some(a.cmp(&(*b as u64))),
            (UInt64(a), UInt16(b)) => Some(a.cmp(&(*b as u64))),
            (UInt64(a), UInt32(b)) => Some(a.cmp(&(*b as u64))),
            (UInt64(a), UInt128(b)) => Some((*a as u128).cmp(b)),
            _ => None,
        }
    }
}

impl<'a> PartialOrd for AnyValue<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> Ord for AnyValue<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        if let Some(like_cmp) = self.fuzzy_cmp(other) {
            like_cmp
        } else {
            match self.variant_rank().cmp(&other.variant_rank()) {
                Ordering::Equal => self.cmp_same_variant(other),
                non_eq => non_eq,
            }
        }
    }
}
