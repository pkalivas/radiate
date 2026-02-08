use crate::Field;
use serde::{Deserialize, Serialize};

pub mod dtype_names {
    pub const NULL: &str = "null";
    pub const BOOLEAN: &str = "boolean";
    pub const UINT8: &str = "uint8";
    pub const UINT16: &str = "uint16";
    pub const UINT32: &str = "uint32";
    pub const UINT64: &str = "uint64";
    pub const UINT128: &str = "uint128";
    pub const INT8: &str = "int8";
    pub const INT16: &str = "int16";
    pub const INT32: &str = "int32";
    pub const INT64: &str = "int64";
    pub const INT128: &str = "int128";
    pub const FLOAT32: &str = "float32";
    pub const FLOAT64: &str = "float64";
    pub const USIZE: &str = "usize";
    pub const BINARY: &str = "binary";
    pub const CHAR: &str = "char";
    pub const STRING: &str = "string";
    pub const DATE: &str = "date";
    pub const VEC: &str = "vec";
    pub const STRUCT: &str = "struct";
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum DataType {
    #[default]
    Null,

    UInt8,
    UInt16,
    UInt32,
    UInt64,
    UInt128,

    Int8,
    Int16,
    Int32,
    Int64,
    Int128,

    Float32,
    Float64,

    Usize,

    Boolean,
    Binary,

    Char,
    Str,
    String,

    Date,
    Datetime,
    DatetimeOwned,

    Vec,
    Struct(Vec<Field>),

    Unknown,
}

impl DataType {
    pub fn is_nested(&self) -> bool {
        use DataType as D;

        matches!(self, D::Vec | D::Struct(_))
    }

    pub fn is_numeric(&self) -> bool {
        use DataType as D;
        matches!(
            self,
            D::Int8
                | D::Int16
                | D::Int32
                | D::Int64
                | D::Int128
                | D::UInt8
                | D::UInt16
                | D::UInt32
                | D::UInt64
                | D::Float32
                | D::Float64
        )
    }

    pub fn is_primitive(&self) -> bool {
        use DataType as D;
        matches!(
            self,
            D::Null
                | D::Boolean
                | D::Int8
                | D::Int16
                | D::Int32
                | D::Int64
                | D::Int128
                | D::UInt8
                | D::UInt16
                | D::UInt32
                | D::UInt64
                | D::Float32
                | D::Float64
                | D::Binary
                | D::Char
                | D::Str
                | D::String
                | D::Vec
        )
    }
}

impl From<String> for DataType {
    fn from(value: String) -> Self {
        match value.as_str() {
            dtype_names::NULL => DataType::Null,

            dtype_names::UINT8 => DataType::UInt8,
            dtype_names::UINT16 => DataType::UInt16,
            dtype_names::UINT32 => DataType::UInt32,
            dtype_names::UINT64 => DataType::UInt64,
            dtype_names::UINT128 => DataType::UInt128,

            dtype_names::INT8 => DataType::Int8,
            dtype_names::INT16 => DataType::Int16,
            dtype_names::INT32 => DataType::Int32,
            dtype_names::INT64 => DataType::Int64,
            dtype_names::INT128 => DataType::Int128,

            dtype_names::FLOAT32 => DataType::Float32,
            dtype_names::FLOAT64 => DataType::Float64,

            dtype_names::BOOLEAN => DataType::Boolean,
            dtype_names::BINARY => DataType::Binary,

            dtype_names::CHAR => DataType::Char,
            dtype_names::STRING => DataType::String,

            dtype_names::VEC => DataType::Vec,
            dtype_names::DATE => DataType::Date,

            _ => DataType::Unknown,
        }
    }
}
