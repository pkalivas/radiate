use super::{DataType, Field};
use crate::Wrap;
use pyo3::{
    Bound, FromPyObject, IntoPyObject, PyAny, PyErr, PyResult, Python, exceptions::PyValueError,
};
use std::{
    fmt::Debug,
    ops::{Add, Div, Mul, Sub},
};

/// Internal helper: perform `lhs <op> rhs` for all numeric AnyValue variants.
/// On type mismatch, returns `AnyValue::Null`.
macro_rules! bin_numeric_op {
    ($lhs:expr, $rhs:expr, $op:tt) => {{
        use AnyValue::*;
        match ($lhs, $rhs) {
            (Int8(a),    Int8(b))    => Int8(a $op b),
            (Int16(a),   Int16(b))   => Int16(a $op b),
            (Int32(a),   Int32(b))   => Int32(a $op b),
            (Int64(a),   Int64(b))   => Int64(a $op b),
            (Int128(a),  Int128(b))  => Int128(a $op b),
            (UInt8(a),   UInt8(b))   => UInt8(a $op b),
            (UInt16(a),  UInt16(b))  => UInt16(a $op b),
            (UInt32(a),  UInt32(b))  => UInt32(a $op b),
            (UInt64(a),  UInt64(b))  => UInt64(a $op b),
            (Float32(a), Float32(b)) => Float32(a $op b),
            (Float64(a), Float64(b)) => Float64(a $op b),
            _ => Null,
        }
    }};
}

/// Like `bin_numeric_op!`, but with integer safe divide (avoid div-by-zero).
macro_rules! bin_numeric_div {
    ($lhs:expr, $rhs:expr) => {{
        use AnyValue::*;
        match ($lhs, $rhs) {
            (Int8(a), Int8(b)) => Int8(if b == 0 { a } else { a / b }),
            (Int16(a), Int16(b)) => Int16(if b == 0 { a } else { a / b }),
            (Int32(a), Int32(b)) => Int32(if b == 0 { a } else { a / b }),
            (Int64(a), Int64(b)) => Int64(if b == 0 { a } else { a / b }),
            (Int128(a), Int128(b)) => Int128(if b == 0 { a } else { a / b }),
            (UInt8(a), UInt8(b)) => UInt8(if b == 0 { a } else { a / b }),
            (UInt16(a), UInt16(b)) => UInt16(if b == 0 { a } else { a / b }),
            (UInt32(a), UInt32(b)) => UInt32(if b == 0 { a } else { a / b }),
            (UInt64(a), UInt64(b)) => UInt64(if b == 0 { a } else { a / b }),
            (Float32(a), Float32(b)) => Float32(a / b), // IEEE handles inf/NaN
            (Float64(a), Float64(b)) => Float64(a / b),
            _ => Null,
        }
    }};
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
    Vec(Box<Vec<AnyValue<'a>>>),
    Struct(Vec<(AnyValue<'a>, Field)>),
}

impl<'a> AnyValue<'a> {
    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }

    pub fn is_boolean(&self) -> bool {
        matches!(self, Self::Bool(_))
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
            Self::Vec(_) => "list",
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
            Self::Vec(_) => DataType::Vec,
            Self::StrOwned(_) => DataType::String,
            Self::Binary(_) => DataType::BinaryView,
            Self::Struct(fields) => DataType::Struct(
                fields
                    .iter()
                    .map(|(_, field)| field.clone())
                    .collect::<Vec<_>>(),
            ),
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
            Vec(v) => Vec(Box::new(
                v.iter().cloned().map(AnyValue::into_static).collect(),
            )),
            StrOwned(v) => StrOwned(v),
            Binary(v) => Binary(v),
            Struct(v) => Struct(
                v.into_iter()
                    .map(|(val, field)| (val.into_static(), field))
                    .collect(),
            ),
        }
    }

    pub fn map<'b, F>(&'b self, f: F) -> AnyValue<'b>
    where
        F: Fn(&'b AnyValue<'b>) -> AnyValue<'b>,
        'b: 'a,
    {
        match self {
            AnyValue::Vec(v) => AnyValue::Vec(Box::new(v.iter().map(f).collect())),
            AnyValue::Struct(v) => AnyValue::Struct(
                v.iter()
                    .map(|(val, field)| (f(val), field.clone()))
                    .collect(),
            ),
            _ => f(self),
        }
    }
}

impl Add for AnyValue<'_> {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        let is_numeric = self.dtype().is_numeric() && other.dtype().is_numeric();
        let is_nested = self.dtype().is_nested() && other.dtype().is_nested();

        if !is_numeric && !is_nested {
            return self;
        }

        match (self, other) {
            // your special Bool policy for "add"
            (AnyValue::Bool(a), AnyValue::Bool(b)) => AnyValue::Bool(a && b),

            // element-wise recursion for nested values
            (AnyValue::Vec(a), AnyValue::Vec(b)) => AnyValue::Vec(Box::new(
                a.iter()
                    .cloned()
                    .zip(b.iter().cloned())
                    .map(|(x, y)| x + y)
                    .collect(),
            )),
            (AnyValue::Struct(a), AnyValue::Struct(b)) => AnyValue::Struct(
                a.iter()
                    .zip(b.iter())
                    .map(|((v1, f1), (v2, f2))| {
                        assert_eq!(f1.name(), f2.name());
                        (v1.clone() + v2.clone(), f1.clone())
                    })
                    .collect(),
            ),

            // numeric variants (same-typed)
            (lhs, rhs) => bin_numeric_op!(lhs, rhs, +),
        }
    }
}

impl Sub for AnyValue<'_> {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        let is_numeric = self.dtype().is_numeric() && other.dtype().is_numeric();
        let is_nested = self.dtype().is_nested() && other.dtype().is_nested();

        if !is_numeric && !is_nested {
            return self;
        }

        match (self, other) {
            (AnyValue::Vec(a), AnyValue::Vec(b)) => AnyValue::Vec(Box::new(
                a.iter()
                    .cloned()
                    .zip(b.iter().cloned())
                    .map(|(x, y)| x - y)
                    .collect(),
            )),
            (AnyValue::Struct(a), AnyValue::Struct(b)) => AnyValue::Struct(
                a.iter()
                    .zip(b.iter())
                    .map(|((v1, f1), (v2, f2))| {
                        assert_eq!(f1.name(), f2.name());
                        (v1.clone() - v2.clone(), f1.clone())
                    })
                    .collect(),
            ),
            (lhs, rhs) => bin_numeric_op!(lhs, rhs, -),
        }
    }
}

impl Mul for AnyValue<'_> {
    type Output = Self;
    fn mul(self, other: Self) -> Self {
        let is_numeric = self.dtype().is_numeric() && other.dtype().is_numeric();
        let is_nested = self.dtype().is_nested() && other.dtype().is_nested();

        if !is_numeric && !is_nested {
            return self;
        }

        match (self, other) {
            (AnyValue::Vec(a), AnyValue::Vec(b)) => AnyValue::Vec(Box::new(
                a.iter()
                    .cloned()
                    .zip(b.iter().cloned())
                    .map(|(x, y)| x * y)
                    .collect(),
            )),
            (AnyValue::Struct(a), AnyValue::Struct(b)) => AnyValue::Struct(
                a.iter()
                    .zip(b.iter())
                    .map(|((v1, f1), (v2, f2))| {
                        assert_eq!(f1.name(), f2.name());
                        (v1.clone() * v2.clone(), f1.clone())
                    })
                    .collect(),
            ),
            (lhs, rhs) => bin_numeric_op!(lhs, rhs, *),
        }
    }
}

impl Div for AnyValue<'_> {
    type Output = Self;
    fn div(self, other: Self) -> Self {
        let is_numeric = self.dtype().is_numeric() && other.dtype().is_numeric();
        let is_nested = self.dtype().is_nested() && other.dtype().is_nested();

        if !is_numeric && !is_nested {
            return self;
        }

        match (self, other) {
            (AnyValue::Vec(a), AnyValue::Vec(b)) => AnyValue::Vec(Box::new(
                a.iter()
                    .cloned()
                    .zip(b.iter().cloned())
                    .map(|(x, y)| x / y)
                    .collect(),
            )),
            (AnyValue::Struct(a), AnyValue::Struct(b)) => AnyValue::Struct(
                a.iter()
                    .zip(b.iter())
                    .map(|((v1, f1), (v2, f2))| {
                        assert_eq!(f1.name(), f2.name());
                        (v1.clone() / v2.clone(), f1.clone())
                    })
                    .collect(),
            ),
            (lhs, rhs) => bin_numeric_div!(lhs, rhs),
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
            (Vec(a), Vec(b)) if a.len() == b.len() => a.iter().zip(b.iter()).all(|(x, y)| x == y),
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

/// One function to “mean” any two AnyValue trees.

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
        Wrap(self.0.clone()).into_pyobject(py)
    }
}
