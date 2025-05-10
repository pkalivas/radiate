use super::{DataType, Field};

#[derive(Debug, Clone, Default, PartialEq)]
pub enum AnyValue<'a> {
    #[default]
    Null,
    /// A binary true or false.
    Boolean(bool),
    /// A UTF8 encoded string type.
    String(&'a str),
    /// An unsigned 8-bit integer number.
    UInt8(u8),
    /// An unsigned 16-bit integer number.
    UInt16(u16),
    /// An unsigned 32-bit integer number.
    UInt32(u32),
    /// An unsigned 64-bit integer number.
    UInt64(u64),
    /// An 8-bit integer number.
    Int8(i8),
    /// A 16-bit integer number.
    Int16(i16),
    /// A 32-bit integer number.
    Int32(i32),
    /// A 64-bit integer number.
    Int64(i64),
    /// A 128-bit integer number.
    Int128(i128),
    /// A 32-bit floating point number.
    Float32(f32),
    /// A 64-bit floating point number.
    Float64(f64),

    Char(char),

    Slice(&'a [AnyValue<'a>], &'a Field),

    VecOwned(Box<(Vec<AnyValue<'a>>, Field)>),

    StringOwned(String),

    Binary(&'a [u8]),

    BinaryOwned(Vec<u8>),

    StructOwned(Box<(Vec<AnyValue<'a>>, Vec<Field>)>),
}

impl<'a> AnyValue<'a> {
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::Null => "null",
            Self::Boolean(_) => "bool",
            Self::String(_) => "string",
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
            Self::Slice(_, _) => "list",
            Self::VecOwned(_) => "list",
            Self::StringOwned(_) => "string",
            Self::Binary(_) => "binary",
            Self::BinaryOwned(_) => "binary",
            Self::StructOwned(_) => "struct",
        }
    }

    pub fn dtype(&self) -> DataType {
        match self {
            Self::Null => DataType::Null,
            Self::Boolean(_) => DataType::Boolean,
            Self::String(_) => DataType::Utf8,
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
            Self::Slice(_, flds) => DataType::List(Box::new((*flds).clone())),
            Self::VecOwned(vals) => DataType::List(Box::new(Field::new(
                vals.1.name().clone(),
                vals.1.dtype().clone(),
            ))),
            Self::StringOwned(_) | Self::BinaryOwned(_) | Self::Binary(_) => DataType::Binary,
            Self::StructOwned(vals) => DataType::Struct(vals.1.iter().cloned().collect::<Vec<_>>()),
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
            Boolean(v) => Boolean(v),
            Float32(v) => Float32(v),
            Float64(v) => Float64(v),
            Char(v) => Char(v),
            String(v) => StringOwned(v.to_string()),
            Slice(v, f) => VecOwned(Box::new((
                v.iter().cloned().map(AnyValue::into_static).collect(),
                f.clone(),
            ))),
            VecOwned(v) => VecOwned(Box::new((
                v.0.iter().cloned().map(AnyValue::into_static).collect(),
                v.1.clone(),
            ))),
            StringOwned(v) => StringOwned(v),
            Binary(v) => BinaryOwned(v.to_vec()),
            BinaryOwned(v) => BinaryOwned(v),
            StructOwned(v) => StructOwned(Box::new((
                v.0.iter().cloned().map(AnyValue::into_static).collect(),
                v.1.iter().cloned().collect(),
            ))),
        }
    }
}

impl<'a, T> From<Vec<T>> for AnyValue<'a>
where
    T: Into<AnyValue<'a>>,
{
    fn from(value: Vec<T>) -> Self {
        Self::VecOwned(Box::new((
            value.into_iter().map(Into::into).collect(),
            Field::new(
                std::any::type_name::<Vec<T>>().to_string(),
                DataType::List(Box::new(Field::new("item".to_string(), DataType::Null))),
            ),
        )))
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
    bool => Boolean,
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
    String => StringOwned,
    char => Char,
    &'a str => String,

);
