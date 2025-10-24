use std::{fmt::Display, sync::Arc};

#[derive(Debug, Clone, Eq, PartialEq, Hash, Default)]
pub struct Field {
    pub name: Arc<String>,
}

impl Field {
    pub fn new(name: String) -> Self {
        Field {
            name: Arc::new(name),
        }
    }

    #[inline]
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }
}

impl From<&str> for Field {
    fn from(s: &str) -> Self {
        Self::new(s.to_string())
    }
}

impl From<String> for Field {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl Display for Field {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Field {{\n name: {},\n }}", self.name())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum DataType {
    #[default]
    Null,
    Boolean,
    Int8,
    Int16,
    Int32,
    Int64,
    Int128,
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    Float16,
    Float32,
    Float64,
    BinaryView,
    Char,
    StringView,
    String,
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
                | D::Float16
                | D::Float32
                | D::Float64
                | D::BinaryView
                | D::Char
                | D::StringView
                | D::String
                | D::Vec
        )
    }
}

impl From<String> for DataType {
    fn from(value: String) -> Self {
        match value.as_str() {
            "null" => DataType::Null,
            "boolean" => DataType::Boolean,
            "int8" => DataType::Int8,
            "int16" => DataType::Int16,
            "int32" => DataType::Int32,
            "int64" => DataType::Int64,
            "int128" => DataType::Int128,
            "uint8" => DataType::UInt8,
            "uint16" => DataType::UInt16,
            "uint32" => DataType::UInt32,
            "uint64" => DataType::UInt64,
            "float16" => DataType::Float16,
            "float32" => DataType::Float32,
            "float64" => DataType::Float64,
            "binary" => DataType::BinaryView,
            "char" => DataType::Char,
            "string" => DataType::String,
            "vec" => DataType::Vec,
            _ => DataType::Unknown,
        }
    }
}
