use super::Field;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum DataType {
    /// Null type
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
    VecView,
    Vec,
    StructView(Vec<Field>),
    Struct(Vec<Field>),
    Object,
    Unknown,
}

impl DataType {
    pub fn is_nested(&self) -> bool {
        use DataType as D;

        matches!(self, D::Vec | D::VecView | D::Struct(_) | D::StructView(_))
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
                | D::VecView
                | D::Vec
        )
    }
}
