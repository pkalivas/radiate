use super::{DataType, Field};
use crate::Wrap;
use pyo3::{
    Bound, FromPyObject, IntoPyObject, PyAny, PyErr, PyResult, Python, exceptions::PyValueError,
};
use std::fmt::Debug;

/// A mutable view into an `AnyValue` that is known to be numeric.
/// This lets callers mutate in-place in the slot's native dtype.
#[derive(Debug)]
pub enum NumericSlotMut<'a> {
    F32(&'a mut f32),
    F64(&'a mut f64),
    U8(&'a mut u8),
    U16(&'a mut u16),
    U32(&'a mut u32),
    U64(&'a mut u64),
    I8(&'a mut i8),
    I16(&'a mut i16),
    I32(&'a mut i32),
    I64(&'a mut i64),
    I128(&'a mut i128),
}

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
    pub fn is_nested(&self) -> bool {
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
    pub fn get_struct_value(&self, name: &str) -> Option<&AnyValue<'a>> {
        if let Some(fields) = self.struct_fields() {
            for (value, field) in fields {
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
        if let Some(fields) = self.struct_fields_mut() {
            for (value, field) in fields {
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
    pub fn numeric_mut(&mut self) -> Option<NumericSlotMut<'_>> {
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
    pub fn with_numeric_mut<R>(&mut self, f: impl FnOnce(NumericSlotMut<'_>) -> R) -> Option<R> {
        self.numeric_mut().map(f)
    }

    //     use crate::any::value::{apply_numeric_slot_mut, NumericSlotMut};

    // allele.with_struct_field_numeric_mut("x", |slot| {
    //     apply_numeric_slot_mut(
    //         slot,
    //         |f| f + jitter_f32,                    // floats
    //         |f| f + jitter_f64,                    // doubles
    //         |i, unsigned| {
    //             let delta: i128 = gaussian_round as i128;
    //             let y = i.saturating_add(delta);
    //             if unsigned { y.max(0) } else { y }
    //         },
    //     );
    // });
    #[inline]
    pub fn with_struct_field_numeric_mut<R>(
        &mut self,
        field_name: &str,
        f: impl FnOnce(NumericSlotMut<'_>) -> R,
    ) -> bool {
        if let Some(slot) = self.get_struct_value_mut(field_name) {
            if let Some(n) = slot.numeric_mut() {
                let _ = f(n);
                return true;
            }
        }
        false
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

#[inline]
pub(crate) fn zip_vec_any_value_apply(
    one: &[AnyValue<'_>],
    two: &[AnyValue<'_>],
    f: impl Fn(&AnyValue<'_>, &AnyValue<'_>) -> Option<AnyValue<'static>>,
) -> Option<AnyValue<'static>> {
    if one.len() != two.len() {
        return None;
    }

    let mut out = Vec::with_capacity(one.len());
    for (x, y) in one.iter().zip(two) {
        out.push(f(x, y)?);
    }

    Some(AnyValue::Vector(Box::new(out)))
}

#[inline]
pub(crate) fn zip_struct_any_value_apply(
    one: &[(AnyValue<'_>, Field)],
    two: &[(AnyValue<'_>, Field)],
    f: impl Fn(&AnyValue<'_>, &AnyValue<'_>) -> Option<AnyValue<'static>>,
) -> Option<AnyValue<'static>> {
    if one.len() != two.len() {
        return None;
    }

    if !one
        .iter()
        .map(|(_, f)| f.name())
        .eq(two.iter().map(|(_, f)| f.name()))
    {
        return None;
    }

    let mut out = Vec::with_capacity(one.len());
    for ((va, fa), (vb, fb)) in one.iter().zip(two.iter()) {
        debug_assert_eq!(fa.name(), fb.name());
        out.push((f(va, vb)?, fa.clone()));
    }

    Some(AnyValue::Struct(out))
}

/// Apply separate integer/float mutators to a numeric slot.
/// - `f_f32`/`f_f64` are used for floats
/// - `f_i` is used for all integer widths; `unsigned=true` for U* variants
pub(crate) fn apply_numeric_slot_mut(
    slot: NumericSlotMut<'_>,
    mut f_f32: impl FnMut(f32) -> f32,
    mut f_f64: impl FnMut(f64) -> f64,
    mut f_i: impl FnMut(i128, bool) -> i128,
) {
    match slot {
        NumericSlotMut::F32(v) => *v = f_f32(*v),
        NumericSlotMut::F64(v) => *v = f_f64(*v),
        NumericSlotMut::U8(v) => *v = f_i(*v as i128, true).max(0).min(u8::MAX as i128) as u8,
        NumericSlotMut::U16(v) => *v = f_i(*v as i128, true).max(0).min(u16::MAX as i128) as u16,
        NumericSlotMut::U32(v) => *v = f_i(*v as i128, true).max(0).min(u32::MAX as i128) as u32,
        NumericSlotMut::U64(v) => *v = f_i(*v as i128, true).max(0).min(u64::MAX as i128) as u64,
        NumericSlotMut::I8(v) => {
            *v = f_i(*v as i128, false).clamp(i8::MIN as i128, i8::MAX as i128) as i8
        }
        NumericSlotMut::I16(v) => {
            *v = f_i(*v as i128, false).clamp(i16::MIN as i128, i16::MAX as i128) as i16
        }
        NumericSlotMut::I32(v) => {
            *v = f_i(*v as i128, false).clamp(i32::MIN as i128, i32::MAX as i128) as i32
        }
        NumericSlotMut::I64(v) => {
            *v = f_i(*v as i128, false).clamp(i64::MIN as i128, i64::MAX as i128) as i64
        }
        NumericSlotMut::I128(v) => *v = f_i(*v as i128, false),
    }
}
