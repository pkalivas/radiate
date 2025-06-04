use super::{DataType, Field};
use std::fmt::Debug;

#[derive(Clone, Default, Debug)]
pub enum AnyValue<'a> {
    #[default]
    Null,
    Bool(bool),
    UInt8(u8),
    UInt16(u16),
    UInt32(u32),
    UInt64(u64),
    Int8(i8),
    Int16(i16),
    Int32(i32),
    Int64(i64),
    Int128(i128),
    Float32(f32),
    Float64(f64),
    Binary(Vec<u8>),
    Char(char),
    Str(&'a str),
    StrOwned(String),
    Vec(Box<Vec<AnyValue<'a>>>),
    Struct(Vec<(AnyValue<'a>, Field)>),
}

impl<'a> AnyValue<'a> {
    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }

    pub fn is_boolean(&self) -> bool {
        matches!(self, Self::Bool(_))
    }

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

    pub fn type_name(&self) -> &'static str {
        match self {
            Self::Null => "null",
            Self::Bool(_) => "bool",
            Self::Str(_) => "string",
            Self::UInt8(_) => "u8",
            Self::UInt16(_) => "u16",
            Self::UInt32(_) => "u32",
            Self::UInt64(_) => "u64",
            Self::Int8(_) => "i8",
            Self::Int16(_) => "i16",
            Self::Int32(_) => "i32",
            Self::Int64(_) => "i64",
            Self::Int128(_) => "i128",
            Self::Float32(_) => "f32",
            Self::Float64(_) => "f64",
            Self::Char(_) => "char",
            Self::Vec(_) => "list",
            Self::StrOwned(_) => "string",
            Self::Binary(_) => "binary",
            Self::Struct(_) => "struct",
        }
    }

    pub fn dtype(&self) -> DataType {
        match self {
            Self::Null => DataType::Null,
            Self::Bool(_) => DataType::Boolean,
            Self::Str(_) => DataType::StringView,
            Self::UInt8(_) => DataType::UInt8,
            Self::UInt16(_) => DataType::UInt16,
            Self::UInt32(_) => DataType::UInt32,
            Self::UInt64(_) => DataType::UInt64,
            Self::Int8(_) => DataType::Int8,
            Self::Int16(_) => DataType::Int16,
            Self::Int32(_) => DataType::Int32,
            Self::Int64(_) => DataType::Int64,
            Self::Int128(_) => DataType::Int128,
            Self::Float32(_) => DataType::Float32,
            Self::Float64(_) => DataType::Float64,
            Self::Char(_) => DataType::Char,
            Self::Vec(_) => DataType::Vec,
            Self::StrOwned(_) => DataType::String,
            Self::Binary(_) => DataType::BinaryView,
            Self::Struct(fields) => DataType::Struct(
                fields
                    .iter()
                    .map(|(_, field)| field.clone())
                    .collect::<Vec<_>>(),
            ),
        }
    }

    /// Try to coerce to an AnyValue with static lifetime.
    /// This can be done if it does not borrow any values.
    #[inline]
    pub fn into_static(self) -> AnyValue<'static> {
        use AnyValue::*;
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
            Bool(v) => Bool(v),
            Float32(v) => Float32(v),
            Float64(v) => Float64(v),
            Char(v) => Char(v),
            Str(v) => StrOwned(v.to_string()),
            Vec(v) => Vec(Box::new(
                v.iter().cloned().map(AnyValue::into_static).collect(),
            )),
            StrOwned(v) => StrOwned(v),
            Binary(v) => Binary(v),
            Struct(v) => Struct(
                v.into_iter()
                    .map(|(val, field)| (val.into_static(), field))
                    .collect(),
            ),
        }
    }
}

impl<'a> PartialEq for AnyValue<'a> {
    fn eq(&self, other: &Self) -> bool {
        use AnyValue::*;
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
            (Binary(a), Binary(b)) => a == b,
            (Vec(a), Vec(b)) if a.len() == b.len() => a.iter().zip(b.iter()).all(|(x, y)| x == y),
            (Struct(a), Struct(b))
                if a.len() == b.len()
                    && a.iter()
                        .map(|(_, f)| f.name())
                        .eq(b.iter().map(|(_, f)| f.name())) =>
            {
                a.iter()
                    .zip(b.iter())
                    .all(|((v1, f1), (v2, f2))| f1.name() == f2.name() && v1 == v2)
            }
            _ => false,
        }
    }
}

impl<'a, T> From<Vec<T>> for AnyValue<'a>
where
    T: Into<AnyValue<'a>>,
{
    fn from(value: Vec<T>) -> Self {
        Self::Vec(Box::new(value.into_iter().map(Into::into).collect()))
    }
}

macro_rules! impl_from {
    ($($dtype:ty => $variant:ident),+ $(,)?) => {
        $(
            impl<'a> From<$dtype> for AnyValue<'a> {
                fn from(value: $dtype) -> Self {
                    Self::$variant(value)
                }
            }
        )*
    };
}

impl_from!(
    bool => Bool,
    u8 => UInt8,
    u16 => UInt16,
    u32 => UInt32,
    u64 => UInt64,
    i8 => Int8,
    i16 => Int16,
    i32 => Int32,
    i64 => Int64,
    i128 => Int128,
    f32 => Float32,
    f64 => Float64,
    String => StrOwned,
    char => Char,
    &'a str => Str,
);
