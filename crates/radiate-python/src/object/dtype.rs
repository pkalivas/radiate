use super::Field;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum DataType {
    /// Null type
    #[default]
    Null,
    /// `true` and `false`.
    Boolean,
    /// An [`i8`]
    Int8,
    /// An [`i16`]
    Int16,
    /// An [`i32`]
    Int32,
    /// An [`i64`]
    Int64,
    /// An [`i128`]
    Int128,
    /// An [`u8`]
    UInt8,
    /// An [`u16`]
    UInt16,
    /// An [`u32`]
    UInt32,
    /// An [`u64`]
    UInt64,
    /// An 16-bit float
    Float16,
    /// A [`f32`]
    Float32,
    /// A [`f64`]
    Float64,
    Char,
    /// Opaque binary data of variable length whose offsets are represented as [`i32`].
    Binary,
    /// Opaque binary data of fixed size.
    /// Enum parameter specifies the number of bytes per value.
    FixedSizeBinary(usize),
    /// Opaque binary data of variable length whose offsets are represented as [`i64`].
    LargeBinary,
    /// A variable-length UTF-8 encoded string whose offsets are represented as [`i32`].
    Utf8,
    /// A variable-length UTF-8 encoded string whose offsets are represented as [`i64`].
    LargeUtf8,
    /// A list of some logical data type whose offsets are represented as [`i32`].
    List(Box<Field>),
    /// A list of some logical data type with a fixed number of elements.
    FixedSizeList(Box<Field>, usize),
    /// A list of some logical data type whose offsets are represented as [`i64`].
    LargeList(Box<Field>),
    /// A nested [`ArrowDataType`] with a given number of [`Field`]s.
    Struct(Vec<Field>),
    /// A nested type that is represented as
    ///
    /// List<entries: Struct<key: K, value: V>>
    ///
    /// In this layout, the keys and values are each respectively contiguous. We do
    /// not constrain the key and value types, so the application is responsible
    /// for ensuring that the keys are hashable and unique. Whether the keys are sorted
    /// may be set in the metadata for this field.
    ///
    /// In a field with Map type, the field has a child Struct field, which then
    /// has two children: key type and the second the value type. The names of the
    /// child fields may be respectively "entries", "key", and "value", but this is
    /// not enforced.
    ///
    /// Map
    /// ```text
    ///   - child[0] entries: Struct
    ///     - child[0] key: K
    ///     - child[1] value: V
    /// ```
    /// Neither the "entries" field nor the "key" field may be nullable.
    ///
    /// The metadata is structured so that Arrow systems without special handling
    /// for Map can make Map an alias for List. The "layout" attribute for the Map
    /// field must have the same contents as a List.
    /// - Field
    /// - ordered
    Map(Box<Field>, bool),
    /// Decimal value with precision and scale
    /// precision is the number of digits in the number and
    /// scale is the number of decimal places.
    /// The number 999.99 has a precision of 5 and scale of 2.
    Decimal(usize, usize),
    /// A string type that inlines small values
    /// and can intern strings.
    Utf8View,
    /// A type unknown to Arrow.
    Unknown,
}

impl DataType {
    pub fn inner_dtype(&self) -> Option<&DataType> {
        match self {
            DataType::List(inner) => Some(inner.dtype()),
            DataType::LargeList(inner) => Some(inner.dtype()),
            DataType::FixedSizeList(inner, _) => Some(inner.dtype()),
            _ => None,
        }
    }

    pub fn is_nested(&self) -> bool {
        use DataType as D;

        matches!(
            self,
            D::List(_) | D::LargeList(_) | D::FixedSizeList(_, _) | D::Struct(_) | D::Map(_, _)
        )
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
                | D::Decimal(_, _)
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
                | D::Binary
                | D::FixedSizeBinary(_)
                | D::LargeBinary
                | D::Utf8
                | D::LargeUtf8
        )
    }
}
