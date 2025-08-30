use super::{DataType, Field};
use crate::Wrap;
use pyo3::{
    Bound, FromPyObject, IntoPyObject, PyAny, PyErr, PyResult, Python, exceptions::PyValueError,
};
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
    Vector(Box<Vec<AnyValue<'a>>>),
    Struct(Vec<(AnyValue<'a>, Field)>),
}

impl<'a> AnyValue<'a> {
    #[inline]
    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }

    #[inline]
    pub fn is_boolean(&self) -> bool {
        matches!(self, Self::Bool(_))
    }

    #[inline]
    pub fn is_struct(&self) -> bool {
        matches!(self, Self::Struct(_))
    }

    #[inline]
    pub fn struct_fields(&self) -> Option<&[(AnyValue<'a>, Field)]> {
        match self {
            AnyValue::Struct(v) => Some(v.as_slice()),
            _ => None,
        }
    }

    #[inline]
    pub fn struct_fields_mut(&mut self) -> Option<&mut [(AnyValue<'a>, Field)]> {
        match self {
            AnyValue::Struct(v) => Some(v.as_mut_slice()),
            _ => None,
        }
    }

    #[inline]
    pub fn get_field(&self, name: &str) -> Option<&AnyValue<'a>> {
        if let Some(fields) = self.struct_fields() {
            for (value, field) in fields {
                if field.name() == name {
                    return Some(value);
                }
            }
        }

        None
    }

    pub fn get_field_mut(&mut self, name: &str) -> Option<&mut AnyValue<'a>> {
        if let Some(fields) = self.struct_fields_mut() {
            for (value, field) in fields {
                if field.name() == name {
                    return Some(value);
                }
            }
        }

        None
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
            Self::Vector(_) => "list",
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
            Self::Vector(_) => DataType::Vec,
            Self::StrOwned(_) => DataType::String,
            Self::Binary(_) => DataType::BinaryView,
            Self::Struct(_) => {
                let fields = self.struct_fields().expect("covered by match arms above");
                DataType::Struct(fields.iter().map(|(_, f)| f.clone()).collect())
            }
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
            Vector(v) => Vector(Box::new(v.into_iter().map(AnyValue::into_static).collect())),
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
            (Vector(a), Vector(b)) if a.len() == b.len() => {
                a.iter().zip(b.iter()).all(|(x, y)| x == y)
            }
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

impl<'py> FromPyObject<'py> for Wrap<AnyValue<'py>> {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        super::py_object_to_any_value(ob, true).map_err(|e| {
            PyValueError::new_err(format!(
                "{e}\n\nHint: Try setting `strict=False` to allow passing data with mixed types."
            ))
        })
        .map(Wrap)
    }
}

impl<'py> IntoPyObject<'py> for Wrap<AnyValue<'_>> {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        super::any_value_into_py_object(self.0, py)
    }
}

impl<'py> IntoPyObject<'py> for &Wrap<AnyValue<'_>> {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        self.clone().into_pyobject(py)
    }
}

impl<'py> IntoPyObject<'py> for Wrap<&AnyValue<'_>> {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        super::any_value_into_py_object_ref(self.0, py)
    }
}

pub(crate) fn merge_any_values<'a>(base: &mut AnyValue<'a>, patch: AnyValue<'a>) {
    use AnyValue::*;
    match (base, patch) {
        (Struct(base_map), Struct(patch_map)) => {
            for (k, v_patch) in patch_map.into_iter() {
                if let Some((v_base, _)) = base_map
                    .iter_mut()
                    .find(|(_, f)| f.name() == v_patch.name())
                {
                    merge_any_values(v_base, k);
                } else {
                    base_map.push((k, v_patch));
                }
            }
        }
        (slot @ _, v_patch) => {
            *slot = v_patch;
        }
    }
}

pub(crate) fn set_any_value_at_field<'a>(
    allele: &mut AnyValue<'a>,
    field_name: &str,
    new_value: AnyValue<'a>,
) {
    if let Some(field) = allele.get_field_mut(field_name) {
        *field = new_value;
    }
}
