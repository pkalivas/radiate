use num_traits::NumCast;
use radiate_utils::intern;
use std::{fmt::Debug, hash::Hash, time::Duration};

#[derive(Clone, Default, Debug)]
pub enum ExprValue<'a> {
    #[default]
    Null,

    Bool(bool),

    UInt8(u8),
    UInt16(u16),
    UInt32(u32),
    UInt64(u64),
    UInt128(u128),

    Int8(i8),
    Int16(i16),
    Int32(i32),
    Int64(i64),
    Int128(i128),

    Float32(f32),
    Float64(f64),

    Char(char),
    Str(&'a str),
    StrOwned(String),

    Duration(Duration),

    Vector(Vec<ExprValue<'a>>),
    Slice(&'a [ExprValue<'a>]),

    Struct(Vec<(&'a str, ExprValue<'a>)>),
}

impl<'a> ExprValue<'a> {
    #[inline]
    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }

    #[inline]
    pub fn is_boolean(&self) -> bool {
        matches!(self, Self::Bool(_))
    }

    #[inline]
    pub fn is_nested(&self) -> bool {
        matches!(self, Self::Vector(_) | Self::Slice(_))
    }

    #[inline]
    pub fn is_numeric(&self) -> bool {
        matches!(
            self,
            Self::UInt8(_)
                | Self::UInt16(_)
                | Self::UInt32(_)
                | Self::UInt64(_)
                | Self::Int8(_)
                | Self::Int16(_)
                | Self::Int32(_)
                | Self::Int64(_)
                | Self::Int128(_)
                | Self::Float32(_)
                | Self::Float64(_)
        )
    }

    #[inline]
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::Null => "null",
            Self::Bool(_) => "bool",
            Self::UInt8(_) => "u8",
            Self::UInt16(_) => "u16",
            Self::UInt32(_) => "u32",
            Self::UInt64(_) => "u64",
            Self::UInt128(_) => "u128",
            Self::Int8(_) => "i8",
            Self::Int16(_) => "i16",
            Self::Int32(_) => "i32",
            Self::Int64(_) => "i64",
            Self::Int128(_) => "i128",
            Self::Float32(_) => "f32",
            Self::Float64(_) => "f64",
            Self::Char(_) => "char",
            Self::Str(_) => "string",
            Self::StrOwned(_) => "string",

            Self::Duration(_) => "duration",

            Self::Vector(_) => "list",
            Self::Slice(_) => "list",
            Self::Struct(_) => "struct",
        }
    }

    /// Try to coerce to an Value
    ///  with static lifetime.
    /// This can be done if it does not borrow any values.
    #[inline]
    pub fn into_static(self) -> ExprValue<'static> {
        use ExprValue::*;
        match self {
            Null => Null,
            Int8(v) => Int8(v),
            Int16(v) => Int16(v),
            Int32(v) => Int32(v),
            Int64(v) => Int64(v),
            Int128(v) => Int128(v),
            UInt8(v) => UInt8(v),
            UInt16(v) => UInt16(v),
            UInt32(v) => UInt32(v),
            UInt64(v) => UInt64(v),
            UInt128(v) => UInt128(v),
            Bool(v) => Bool(v),
            Float32(v) => Float32(v),
            Float64(v) => Float64(v),
            Char(v) => Char(v),
            Str(v) => StrOwned(v.to_string()),
            StrOwned(v) => StrOwned(v),
            Duration(v) => Duration(v),
            Vector(v) => Vector(v.into_iter().map(ExprValue::into_static).collect()),
            Slice(v) => Vector(v.iter().map(|v| v.clone().into_static()).collect()),
            Struct(v) => Struct(
                v.iter()
                    .map(|(k, v)| (intern!(*k), v.clone().into_static()))
                    .collect(),
            ),
        }
    }

    pub fn as_bool(&self) -> bool {
        match self {
            ExprValue::Bool(v) => *v,
            ExprValue::Null => false,
            ExprValue::UInt8(v) => *v != 0,
            ExprValue::UInt16(v) => *v != 0,
            ExprValue::UInt32(v) => *v != 0,
            ExprValue::UInt64(v) => *v != 0,
            ExprValue::Int8(v) => *v != 0,
            ExprValue::Int16(v) => *v != 0,
            ExprValue::Int32(v) => *v != 0,
            ExprValue::Int64(v) => *v != 0,
            ExprValue::Int128(v) => *v != 0,
            ExprValue::Float32(v) => *v != 0.0,
            ExprValue::Float64(v) => *v != 0.0,
            _ => true, // Non-null non-boolean values are considered true
        }
    }

    pub fn extract<T: NumCast>(self) -> Option<T> {
        match self {
            ExprValue::UInt8(v) => NumCast::from(v),
            ExprValue::UInt16(v) => NumCast::from(v),
            ExprValue::UInt32(v) => NumCast::from(v),
            ExprValue::UInt64(v) => NumCast::from(v),
            ExprValue::UInt128(v) => NumCast::from(v),
            ExprValue::Int8(v) => NumCast::from(v),
            ExprValue::Int16(v) => NumCast::from(v),
            ExprValue::Int32(v) => NumCast::from(v),
            ExprValue::Int64(v) => NumCast::from(v),
            ExprValue::Int128(v) => NumCast::from(v),
            ExprValue::Float32(v) => NumCast::from(v),
            ExprValue::Float64(v) => NumCast::from(v),
            _ => None,
        }
    }
}

impl<'a> PartialEq for ExprValue<'a> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        use ExprValue::*;
        match (self, other) {
            (Null, Null) => true,
            (Bool(a), Bool(b)) => a == b,
            (UInt8(a), UInt8(b)) => a == b,
            (UInt16(a), UInt16(b)) => a == b,
            (UInt32(a), UInt32(b)) => a == b,
            (UInt64(a), UInt64(b)) => a == b,
            (Int8(a), Int8(b)) => a == b,
            (Int16(a), Int16(b)) => a == b,
            (Int32(a), Int32(b)) => a == b,
            (Int64(a), Int64(b)) => a == b,
            (Int128(a), Int128(b)) => a == b,
            (Float32(a), Float32(b)) => a == b,
            (Float64(a), Float64(b)) => a == b,
            (Char(a), Char(b)) => a == b,
            (Str(a), Str(b)) => a == b,
            (StrOwned(a), StrOwned(b)) => a == b,
            (Vector(a), Vector(b)) if a.len() == b.len() => {
                a.iter().zip(b.iter()).all(|(x, y)| x == y)
            }
            (Slice(a), Slice(b)) if a.len() == b.len() => {
                a.iter().zip(b.iter()).all(|(x, y)| x == y)
            }
            (Struct(a), Struct(b)) if a.len() == b.len() => a
                .iter()
                .zip(b.iter())
                .all(|((ka, va), (kb, vb))| ka == kb && va == vb),

            _ => false,
        }
    }
}

impl<'a> Eq for ExprValue<'a> {}

impl<'a> Hash for ExprValue<'a> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        use ExprValue::*;
        match self {
            Null => 0.hash(state),
            Bool(v) => v.hash(state),

            Int8(v) => v.hash(state),
            Int16(v) => v.hash(state),
            Int32(v) => v.hash(state),
            Int64(v) => v.hash(state),
            Int128(v) => v.hash(state),

            UInt8(v) => v.hash(state),
            UInt16(v) => v.hash(state),
            UInt32(v) => v.hash(state),
            UInt64(v) => v.hash(state),
            UInt128(v) => v.hash(state),

            Float32(v) => v.to_ne_bytes().hash(state),
            Float64(v) => v.to_ne_bytes().hash(state),

            Duration(v) => v.hash(state),

            Char(v) => v.hash(state),
            Str(v) => v.hash(state),
            StrOwned(v) => v.hash(state),

            Vector(v) => v.hash(state),
            Slice(v) => v.hash(state),

            Struct(v) => v.iter().for_each(|(k, v)| {
                k.hash(state);
                v.hash(state);
            }),
        }
    }
}

impl<'a> From<&'a str> for ExprValue<'a> {
    fn from(s: &'a str) -> Self {
        ExprValue::Str(s)
    }
}
