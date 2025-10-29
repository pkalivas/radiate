use crate::{
    NumericSlotMut, Wrap,
    any::dtype::{DataType, Field},
};
use pyo3::{
    Borrowed, Bound, FromPyObject, IntoPyObject, PyAny, PyErr, PyResult, Python,
    exceptions::PyValueError,
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
    Struct(Vec<(Field, AnyValue<'a>)>),
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
    pub fn is_nested(&self) -> bool {
        matches!(self, Self::Struct(_) | Self::Vector(_))
    }

    #[inline]
    pub fn struct_as_slice(&self) -> Option<&[(Field, AnyValue<'a>)]> {
        match self {
            AnyValue::Struct(v) => Some(v.as_slice()),
            _ => None,
        }
    }

    #[inline]
    pub fn struct_as_slice_mut(&mut self) -> Option<&mut [(Field, AnyValue<'a>)]> {
        match self {
            AnyValue::Struct(v) => Some(v.as_mut_slice()),
            _ => None,
        }
    }

    #[inline]
    pub fn vec_as_slice(&self) -> Option<&[AnyValue<'a>]> {
        match self {
            AnyValue::Vector(v) => Some(v.as_slice()),
            _ => None,
        }
    }

    #[inline]
    pub fn vec_as_slice_mut(&mut self) -> Option<&mut [AnyValue<'a>]> {
        match self {
            AnyValue::Vector(v) => Some(v.as_mut_slice()),
            _ => None,
        }
    }

    #[inline]
    pub fn get_struct_value(&self, name: &str) -> Option<&AnyValue<'a>> {
        if let Some(fields) = self.struct_as_slice() {
            for (field, value) in fields {
                if value.is_nested() {
                    if let Some(v) = value.get_struct_value(name) {
                        return Some(v);
                    }
                } else if field.name() == name {
                    return Some(value);
                }
            }
        }

        None
    }

    #[inline]
    pub fn get_struct_value_mut(&mut self, name: &str) -> Option<&mut AnyValue<'a>> {
        if let Some(fields) = self.struct_as_slice_mut() {
            for (field, value) in fields {
                if value.is_nested() {
                    if let Some(v) = value.get_struct_value_mut(name) {
                        return Some(v);
                    }
                } else if field.name() == name {
                    return Some(value);
                }
            }
        }

        None
    }

    #[inline]
    pub fn get_numeric_slot_mut(&mut self) -> Option<NumericSlotMut<'_>> {
        use AnyValue::*;
        Some(match self {
            Float32(v) => NumericSlotMut::F32(v),
            Float64(v) => NumericSlotMut::F64(v),
            UInt8(v) => NumericSlotMut::U8(v),
            UInt16(v) => NumericSlotMut::U16(v),
            UInt32(v) => NumericSlotMut::U32(v),
            UInt64(v) => NumericSlotMut::U64(v),
            Int8(v) => NumericSlotMut::I8(v),
            Int16(v) => NumericSlotMut::I16(v),
            Int32(v) => NumericSlotMut::I32(v),
            Int64(v) => NumericSlotMut::I64(v),
            Int128(v) => NumericSlotMut::I128(v),
            _ => return None,
        })
    }

    #[inline]
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

    #[inline]
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

    #[inline]
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
                let fields = self.struct_as_slice().expect("covered by match arms above");
                DataType::Struct(fields.iter().map(|(f, _)| f.clone()).collect())
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
                    .map(|(field, val)| (field, val.into_static()))
                    .collect(),
            ),
        }
    }
}

impl<'a> PartialEq for AnyValue<'a> {
    #[inline]
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
                        .map(|(f, _)| f.name())
                        .eq(b.iter().map(|(f, _)| f.name())) =>
            {
                a.iter()
                    .zip(b.iter())
                    .all(|((f1, v1), (f2, v2))| f1.name() == f2.name() && v1 == v2)
            }
            _ => false,
        }
    }
}

#[inline]
pub(crate) fn apply_zipped_slice(
    one: &[AnyValue<'_>],
    two: &[AnyValue<'_>],
    f: impl Fn(&AnyValue<'_>, &AnyValue<'_>) -> Option<AnyValue<'static>>,
) -> Option<AnyValue<'static>> {
    if one.len() != two.len() {
        return None;
    }

    Some(AnyValue::Vector(Box::new(
        one.iter()
            .zip(two.iter())
            .map(|pair| match f(pair.0, pair.1) {
                Some(v) => v,
                None => AnyValue::Null,
            })
            .collect::<Vec<AnyValue>>(),
    )))
}

#[inline]
pub(crate) fn apply_zipped_struct_slice(
    one: &[(Field, AnyValue<'_>)],
    two: &[(Field, AnyValue<'_>)],
    f: impl Fn(&AnyValue<'_>, &AnyValue<'_>) -> Option<AnyValue<'static>>,
) -> Option<AnyValue<'static>> {
    if one.len() != two.len() {
        return None;
    }

    if !one
        .iter()
        .map(|(f, _)| f.name())
        .eq(two.iter().map(|(f, _)| f.name()))
    {
        return None;
    }

    let mut out = Vec::with_capacity(one.len());
    for ((fa, va), (_, vb)) in one.iter().zip(two.iter()) {
        if va.is_null() || vb.is_null() {
            out.push((fa.clone(), AnyValue::Null));
            continue;
        }

        out.push((fa.clone(), f(va, vb)?));
    }

    Some(AnyValue::Struct(out))
}

impl<'py> FromPyObject<'_, 'py> for Wrap<AnyValue<'py>> {
    type Error = PyErr;

    fn extract(ob: Borrowed<'_, 'py, PyAny>) -> PyResult<Self> {
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
