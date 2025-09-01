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
    Struct(Vec<DataType>),
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

// #[derive(Debug)]
// pub enum NumericSlotMut<'a> {
//     F32(&'a mut f32),
//     F64(&'a mut f64),
//     U8(&'a mut u8),
//     U16(&'a mut u16),
//     U32(&'a mut u32),
//     U64(&'a mut u64),
//     I8(&'a mut i8),
//     I16(&'a mut i16),
//     I32(&'a mut i32),
//     I64(&'a mut i64),
//     I128(&'a mut i128),
// }

// pub(crate) fn apply_numeric_slot_mut(
//     slot: NumericSlotMut<'_>,
//     mut f_f32: impl FnMut(f32) -> f32,
//     mut f_f64: impl FnMut(f64) -> f64,
//     mut f_i: impl FnMut(i128, bool) -> i128,
// ) {
//     match slot {
//         NumericSlotMut::F32(v) => *v = f_f32(*v),
//         NumericSlotMut::F64(v) => *v = f_f64(*v),
//         NumericSlotMut::U8(v) => *v = f_i(*v as i128, true).max(0).min(u8::MAX as i128) as u8,
//         NumericSlotMut::U16(v) => *v = f_i(*v as i128, true).max(0).min(u16::MAX as i128) as u16,
//         NumericSlotMut::U32(v) => *v = f_i(*v as i128, true).max(0).min(u32::MAX as i128) as u32,
//         NumericSlotMut::U64(v) => *v = f_i(*v as i128, true).max(0).min(u64::MAX as i128) as u64,
//         NumericSlotMut::I8(v) => {
//             *v = f_i(*v as i128, false).clamp(i8::MIN as i128, i8::MAX as i128) as i8
//         }
//         NumericSlotMut::I16(v) => {
//             *v = f_i(*v as i128, false).clamp(i16::MIN as i128, i16::MAX as i128) as i16
//         }
//         NumericSlotMut::I32(v) => {
//             *v = f_i(*v as i128, false).clamp(i32::MIN as i128, i32::MAX as i128) as i32
//         }
//         NumericSlotMut::I64(v) => {
//             *v = f_i(*v as i128, false).clamp(i64::MIN as i128, i64::MAX as i128) as i64
//         }
//         NumericSlotMut::I128(v) => *v = f_i(*v as i128, false),
//     }
// }
