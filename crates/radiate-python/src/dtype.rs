use crate::Field;

pub(crate) const NULL: &str = "null";
pub(crate) const BOOLEAN: &str = "boolean";
pub(crate) const UINT8: &str = "uint8";
pub(crate) const UINT16: &str = "uint16";
pub(crate) const UINT32: &str = "uint32";
pub(crate) const UINT64: &str = "uint64";
pub(crate) const UINT128: &str = "uint128";
pub(crate) const INT8: &str = "int8";
pub(crate) const INT16: &str = "int16";
pub(crate) const INT32: &str = "int32";
pub(crate) const INT64: &str = "int64";
pub(crate) const INT128: &str = "int128";
pub(crate) const FLOAT32: &str = "float32";
pub(crate) const FLOAT64: &str = "float64";
pub(crate) const USIZE: &str = "usize";
pub(crate) const BINARY: &str = "binary";
pub(crate) const CHAR: &str = "char";
pub(crate) const STRING: &str = "string";
pub(crate) const DATE: &str = "date";
pub(crate) const VEC: &str = "vec";
pub(crate) const STRUCT: &str = "struct";

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
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
            NULL => DataType::Null,

            UINT8 => DataType::UInt8,
            UINT16 => DataType::UInt16,
            UINT32 => DataType::UInt32,
            UINT64 => DataType::UInt64,
            UINT128 => DataType::UInt128,

            INT8 => DataType::Int8,
            INT16 => DataType::Int16,
            INT32 => DataType::Int32,
            INT64 => DataType::Int64,
            INT128 => DataType::Int128,

            FLOAT32 => DataType::Float32,
            FLOAT64 => DataType::Float64,

            BOOLEAN => DataType::Boolean,
            BINARY => DataType::Binary,

            CHAR => DataType::Char,
            STRING => DataType::String,

            VEC => DataType::Vec,
            DATE => DataType::Date,

            _ => DataType::Unknown,
        }
    }
}
