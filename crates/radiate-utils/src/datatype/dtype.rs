use super::{Field, Scalar};
use crate::Primitive;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::{fmt::Display, sync::Arc};

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
    pub const DURATION: &str = "duration";
    pub const VEC: &str = "vec";
    pub const STRUCT: &str = "struct";
    pub const MAP: &str = "map";
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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

    Duration,

    Boolean,

    Char,
    String,

    List(Box<DataType>),
    Map(Vec<(Arc<String>, DataType)>),
    Struct(Box<Field>, Vec<Field>),
}

impl DataType {
    pub fn is_nested(&self) -> bool {
        use DataType as D;
        matches!(self, D::List(_) | D::Map(_) | D::Struct(_, _))
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
                | D::Usize
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
                | D::Usize
        )
    }

    pub fn max(&self) -> Option<Scalar> {
        use DataType as D;
        match self {
            D::Int8 => Some(Scalar::from(<i8 as Primitive>::MAX)),
            D::Int16 => Some(Scalar::from(<i16 as Primitive>::MAX)),
            D::Int32 => Some(Scalar::from(<i32 as Primitive>::MAX)),
            D::Int64 => Some(Scalar::from(<i64 as Primitive>::MAX)),
            D::Int128 => Some(Scalar::from(<i128 as Primitive>::MAX)),
            D::UInt8 => Some(Scalar::from(<u8 as Primitive>::MAX)),
            D::UInt16 => Some(Scalar::from(<u16 as Primitive>::MAX)),
            D::UInt32 => Some(Scalar::from(<u32 as Primitive>::MAX)),
            D::UInt64 => Some(Scalar::from(<u64 as Primitive>::MAX)),
            D::UInt128 => Some(Scalar::from(<u128 as Primitive>::MAX)),
            D::Float32 => Some(Scalar::from(<f32 as Primitive>::MAX)),
            D::Float64 => Some(Scalar::from(<f64 as Primitive>::MAX)),
            _ => None,
        }
    }

    pub fn min(&self) -> Option<Scalar> {
        use DataType as D;
        match self {
            D::Int8 => Some(Scalar::from(<i8 as Primitive>::MIN)),
            D::Int16 => Some(Scalar::from(<i16 as Primitive>::MIN)),
            D::Int32 => Some(Scalar::from(<i32 as Primitive>::MIN)),
            D::Int64 => Some(Scalar::from(<i64 as Primitive>::MIN)),
            D::Int128 => Some(Scalar::from(<i128 as Primitive>::MIN)),
            D::UInt8 => Some(Scalar::from(<u8 as Primitive>::MIN)),
            D::UInt16 => Some(Scalar::from(<u16 as Primitive>::MIN)),
            D::UInt32 => Some(Scalar::from(<u32 as Primitive>::MIN)),
            D::UInt64 => Some(Scalar::from(<u64 as Primitive>::MIN)),
            D::UInt128 => Some(Scalar::from(<u128 as Primitive>::MIN)),
            D::Float32 => Some(Scalar::from(<f32 as Primitive>::MIN)),
            D::Float64 => Some(Scalar::from(<f64 as Primitive>::MIN)),
            _ => None,
        }
    }

    pub fn primitive_bounds(&self) -> Option<(Scalar, Scalar)> {
        match (self.min(), self.max()) {
            (Some(min), Some(max)) => Some((min, max)),
            _ => None,
        }
    }
}

impl From<String> for DataType {
    fn from(value: String) -> Self {
        match value.trim().to_lowercase().as_str() {
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

            dtype_names::USIZE => DataType::Usize,

            dtype_names::BOOLEAN => DataType::Boolean,

            dtype_names::CHAR => DataType::Char,
            dtype_names::STRING => DataType::String,

            dtype_names::VEC => DataType::List(Box::new(DataType::Null)),
            dtype_names::STRUCT => DataType::Struct(
                Box::new(Field::new("field".into(), DataType::Null)),
                Vec::new(),
            ),

            dtype_names::MAP => DataType::Map(Vec::new()),

            _ => panic!("Unknown data type: {}", value),
        }
    }
}

impl Display for DataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataType::Null => write!(f, "{}", dtype_names::NULL)?,

            DataType::UInt8 => write!(f, "{}", dtype_names::UINT8)?,
            DataType::UInt16 => write!(f, "{}", dtype_names::UINT16)?,
            DataType::UInt32 => write!(f, "{}", dtype_names::UINT32)?,
            DataType::UInt64 => write!(f, "{}", dtype_names::UINT64)?,
            DataType::UInt128 => write!(f, "{}", dtype_names::UINT128)?,

            DataType::Int8 => write!(f, "{}", dtype_names::INT8)?,
            DataType::Int16 => write!(f, "{}", dtype_names::INT16)?,
            DataType::Int32 => write!(f, "{}", dtype_names::INT32)?,
            DataType::Int64 => write!(f, "{}", dtype_names::INT64)?,
            DataType::Int128 => write!(f, "{}", dtype_names::INT128)?,

            DataType::Float32 => write!(f, "{}", dtype_names::FLOAT32)?,
            DataType::Float64 => write!(f, "{}", dtype_names::FLOAT64)?,

            DataType::Usize => write!(f, "{}", dtype_names::USIZE)?,

            DataType::Duration => write!(f, "{}", dtype_names::DURATION)?,

            DataType::Boolean => write!(f, "{}", dtype_names::BOOLEAN)?,

            DataType::Char => write!(f, "{}", dtype_names::CHAR)?,
            DataType::String => write!(f, "{}", dtype_names::STRING)?,

            DataType::List(inner) => write!(f, "{}({})", dtype_names::VEC, inner)?,
            DataType::Map(vals) => write!(
                f,
                "{}({})",
                dtype_names::MAP,
                vals.iter()
                    .map(|(name, _)| format!("{}", name))
                    .collect::<Vec<_>>()
                    .join(", ")
            )?,

            DataType::Struct(name, fields) => write!(
                f,
                "{} {} {{ {} }}",
                dtype_names::STRUCT,
                name.name,
                fields
                    .iter()
                    .map(|f| format!("{}", f.name,))
                    .collect::<Vec<_>>()
                    .join(", ")
            )?,
        };

        Ok(())
    }
}
