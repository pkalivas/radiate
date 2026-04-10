use super::{DataType, Field};
use num_traits::NumCast;
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, hash::Hash, time::Duration};

#[derive(Clone, Default, Debug)]
pub enum AnyValue<'a> {
    #[default]
    Null,

    Bool(bool),

    UInt8(u8),
    UInt16(u16),
    UInt32(u32),
    UInt64(u64),
    UInt128(u128),

    Int8(i8),
    Int16(i16),
    Int32(i32),
    Int64(i64),
    Int128(i128),

    Float32(f32),
    Float64(f64),

    Duration(Duration),

    Char(char),
    Str(&'a str),
    StrOwned(String),

    Slice(&'a [AnyValue<'a>]),
    Vector(Vec<AnyValue<'a>>),

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
        matches!(self, Self::Struct(_) | Self::Vector(_) | Self::Slice(_))
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
            Self::UInt8(_) => "u8",
            Self::UInt16(_) => "u16",
            Self::UInt32(_) => "u32",
            Self::UInt64(_) => "u64",
            Self::UInt128(_) => "u128",
            Self::Int8(_) => "i8",
            Self::Int16(_) => "i16",
            Self::Int32(_) => "i32",
            Self::Int64(_) => "i64",
            Self::Int128(_) => "i128",
            Self::Float32(_) => "f32",
            Self::Float64(_) => "f64",
            Self::Char(_) => "char",
            Self::Str(_) => "string",
            Self::StrOwned(_) => "string",
            Self::Slice(_) => "list",
            Self::Vector(_) => "list",
            Self::Struct(_) => "struct",
            Self::Duration(_) => "duration",
        }
    }

    #[inline]
    pub fn dtype(&self) -> DataType {
        match self {
            Self::Null => DataType::Null,

            Self::Bool(_) => DataType::Boolean,

            Self::UInt8(_) => DataType::UInt8,
            Self::UInt16(_) => DataType::UInt16,
            Self::UInt32(_) => DataType::UInt32,
            Self::UInt64(_) => DataType::UInt64,
            Self::UInt128(_) => DataType::UInt128,

            Self::Int8(_) => DataType::Int8,
            Self::Int16(_) => DataType::Int16,
            Self::Int32(_) => DataType::Int32,
            Self::Int64(_) => DataType::Int64,
            Self::Int128(_) => DataType::Int128,

            Self::Float32(_) => DataType::Float32,
            Self::Float64(_) => DataType::Float64,

            Self::Duration(_) => DataType::Duration,

            Self::Char(_) => DataType::Char,
            Self::Str(_) => DataType::String,
            Self::StrOwned(_) => DataType::String,

            Self::Slice(vals) => DataType::List(
                vals.iter()
                    .map(|v| v.dtype())
                    .next()
                    .unwrap_or(DataType::Null)
                    .into(),
            ),
            Self::Vector(vals) => DataType::List(
                vals.iter()
                    .map(|v| v.dtype())
                    .next()
                    .unwrap_or(DataType::Null)
                    .into(),
            ),

            Self::Struct(vals) => DataType::Struct(vals.iter().map(|(f, _)| f.clone()).collect()),
        }
    }

    pub fn cast(self, to: &DataType) -> Option<AnyValue<'a>> {
        use DataType as D;

        if self.dtype() == *to {
            return Some(self);
        }

        match (self, to) {
            (_, D::Null) => Some(AnyValue::Null),
            (AnyValue::Bool(v), D::Boolean) => Some(AnyValue::Bool(v)),
            (v, D::UInt8) => v.extract().map(AnyValue::UInt8),
            (v, D::UInt16) => v.extract().map(AnyValue::UInt16),
            (v, D::UInt32) => v.extract().map(AnyValue::UInt32),
            (v, D::UInt64) => v.extract().map(AnyValue::UInt64),
            (v, D::UInt128) => v.extract().map(AnyValue::UInt128),
            (v, D::Int8) => v.extract().map(AnyValue::Int8),
            (v, D::Int16) => v.extract().map(AnyValue::Int16),
            (v, D::Int32) => v.extract().map(AnyValue::Int32),
            (v, D::Int64) => v.extract().map(AnyValue::Int64),
            (v, D::Int128) => v.extract().map(AnyValue::Int128),
            (v, D::Float32) => v.extract().map(AnyValue::Float32),
            (v, D::Float64) => v.extract().map(AnyValue::Float64),
            (v, D::Duration) => v
                .extract()
                .map(|ms| AnyValue::Duration(Duration::from_millis(ms))),
            (v, D::Char) => v.extract::<u8>().map(|b| AnyValue::Char(b as char)),
            (v @ AnyValue::Str(_), D::String) | (v @ AnyValue::StrOwned(_), D::String) => {
                Some(v.into_static())
            }
            _ => None,
        }
    }

    pub fn into_string(self) -> Option<String> {
        match self {
            AnyValue::Str(s) => Some(s.to_string()),
            AnyValue::StrOwned(s) => Some(s),
            _ => None,
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
            UInt128(v) => UInt128(v),
            Bool(v) => Bool(v),
            Float32(v) => Float32(v),
            Float64(v) => Float64(v),
            Duration(d) => Duration(d),
            Char(v) => Char(v),
            Str(v) => StrOwned(v.to_string()),
            StrOwned(v) => StrOwned(v),
            Slice(v) => Vector(v.into_iter().map(|v| v.clone().into_static()).collect()),
            Vector(v) => Vector(v.into_iter().map(AnyValue::into_static).collect()),
            Struct(v) => Struct(
                v.into_iter()
                    .map(|(field, val)| (field, val.into_static()))
                    .collect(),
            ),
        }
    }

    pub fn extract<T: NumCast>(self) -> Option<T> {
        match self {
            AnyValue::UInt8(v) => NumCast::from(v),
            AnyValue::UInt16(v) => NumCast::from(v),
            AnyValue::UInt32(v) => NumCast::from(v),
            AnyValue::UInt64(v) => NumCast::from(v),
            AnyValue::UInt128(v) => NumCast::from(v),
            AnyValue::Int8(v) => NumCast::from(v),
            AnyValue::Int16(v) => NumCast::from(v),
            AnyValue::Int32(v) => NumCast::from(v),
            AnyValue::Int64(v) => NumCast::from(v),
            AnyValue::Int128(v) => NumCast::from(v),
            AnyValue::Float32(v) => NumCast::from(v),
            AnyValue::Float64(v) => NumCast::from(v),
            AnyValue::Duration(d) => NumCast::from(d.as_millis() as u64),
            _ => None,
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
            (Duration(a), Duration(b)) => a == b,
            (Char(a), Char(b)) => a == b,
            (Str(a), Str(b)) => a == b,
            (StrOwned(a), StrOwned(b)) => a == b,
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

impl<'a> Eq for AnyValue<'a> {}

impl<'a> Hash for AnyValue<'a> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        use AnyValue::*;
        match self {
            Null => 0.hash(state),
            Bool(v) => v.hash(state),

            Int8(v) => v.hash(state),
            Int16(v) => v.hash(state),
            Int32(v) => v.hash(state),
            Int64(v) => v.hash(state),
            Int128(v) => v.hash(state),

            UInt8(v) => v.hash(state),
            UInt16(v) => v.hash(state),
            UInt32(v) => v.hash(state),
            UInt64(v) => v.hash(state),
            UInt128(v) => v.hash(state),

            Float32(v) => v.to_ne_bytes().hash(state),
            Float64(v) => v.to_ne_bytes().hash(state),

            Duration(v) => v.hash(state),

            Char(v) => v.hash(state),
            Str(v) => v.hash(state),
            StrOwned(v) => v.hash(state),

            Vector(v) => v.hash(state),
            Slice(v) => v.hash(state),

            Struct(v) => v.iter().for_each(|(k, v)| {
                k.hash(state);
                v.hash(state);
            }),
        }
    }
}

impl<'a> From<&'a str> for AnyValue<'a> {
    fn from(s: &'a str) -> Self {
        AnyValue::Str(s)
    }
}

impl<'a> From<String> for AnyValue<'a> {
    fn from(s: String) -> Self {
        AnyValue::StrOwned(s)
    }
}

impl From<f32> for AnyValue<'_> {
    fn from(f: f32) -> Self {
        AnyValue::Float32(f)
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

    Some(AnyValue::Vector(
        one.iter()
            .zip(two.iter())
            .map(|pair| match f(pair.0, pair.1) {
                Some(v) => v,
                None => AnyValue::Null,
            })
            .collect::<Vec<AnyValue>>(),
    ))
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

#[cfg(feature = "serde")]
impl<'a> Serialize for AnyValue<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use AnyValue::*;
        match self {
            Null => serializer.serialize_unit_variant("AnyValue", 0, "Null"),
            Bool(v) => serializer.serialize_bool(*v),
            UInt8(v) => serializer.serialize_u8(*v),
            UInt16(v) => serializer.serialize_u16(*v),
            UInt32(v) => serializer.serialize_u32(*v),
            UInt64(v) => serializer.serialize_u64(*v),
            UInt128(v) => serializer.serialize_u128(*v),
            Int8(v) => serializer.serialize_i8(*v),
            Int16(v) => serializer.serialize_i16(*v),
            Int32(v) => serializer.serialize_i32(*v),
            Int64(v) => serializer.serialize_i64(*v),
            Int128(v) => serializer.serialize_i128(*v),
            Float32(v) => serializer.serialize_f32(*v),
            Float64(v) => serializer.serialize_f64(*v),
            Duration(v) => serializer.serialize_u64(v.as_millis() as u64),
            Char(v) => serializer.serialize_char(*v),
            Str(v) => serializer.serialize_str(v),
            StrOwned(v) => serializer.serialize_str(v),
            Slice(vals) => vals.serialize(serializer),
            Vector(vals) => vals.serialize(serializer),
            Struct(vals) => vals.serialize(serializer),
        }
    }
}

#[cfg(feature = "serde")]
impl<'a, 'de> Deserialize<'de> for AnyValue<'a> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use AnyValue::*;
        #[derive(Deserialize)]
        #[serde(tag = "type", content = "value")]
        enum AnyValueDef {
            Null,
            Bool(bool),
            UInt8(u8),
            UInt16(u16),
            UInt32(u32),
            UInt64(u64),
            UInt128(u128),
            Int8(i8),
            Int16(i16),
            Int32(i32),
            Int64(i64),
            Int128(i128),
            Float32(f32),
            Float64(f64),
            Duration(u64),
            Char(char),
            Str(String),
            StrOwned(String),
            Slice(Vec<AnyValueDef>),
            Vector(Vec<AnyValueDef>),
            Struct(Vec<(Field, AnyValueDef)>),
        }

        impl From<AnyValueDef> for AnyValue<'_> {
            fn from(def: AnyValueDef) -> Self {
                match def {
                    AnyValueDef::Null => Null,
                    AnyValueDef::Bool(v) => Bool(v),
                    AnyValueDef::UInt8(v) => UInt8(v),
                    AnyValueDef::UInt16(v) => UInt16(v),
                    AnyValueDef::UInt32(v) => UInt32(v),
                    AnyValueDef::UInt64(v) => UInt64(v),
                    AnyValueDef::UInt128(v) => UInt128(v),
                    AnyValueDef::Int8(v) => Int8(v),
                    AnyValueDef::Int16(v) => Int16(v),
                    AnyValueDef::Int32(v) => Int32(v),
                    AnyValueDef::Int64(v) => Int64(v),
                    AnyValueDef::Int128(v) => Int128(v),
                    AnyValueDef::Float32(v) => Float32(v),
                    AnyValueDef::Float64(v) => Float64(v),
                    AnyValueDef::Duration(ms) => Duration(std::time::Duration::from_millis(ms)),
                    AnyValueDef::Char(v) => Char(v),
                    AnyValueDef::Str(s) | AnyValueDef::StrOwned(s) => StrOwned(s),
                    AnyValueDef::Slice(vals) => {
                        Vector(vals.into_iter().map(AnyValue::from).collect())
                    }
                    AnyValueDef::Vector(vals) => {
                        Vector(vals.into_iter().map(AnyValue::from).collect())
                    }
                    AnyValueDef::Struct(vals) => {
                        Struct(vals.into_iter().map(|(f, v)| (f, v.into())).collect())
                    }
                }
            }
        }

        let def = AnyValueDef::deserialize(deserializer)?;
        Ok(match def {
            AnyValueDef::Null => Null,
            AnyValueDef::Bool(v) => Bool(v),
            AnyValueDef::UInt8(v) => UInt8(v),
            AnyValueDef::UInt16(v) => UInt16(v),
            AnyValueDef::UInt32(v) => UInt32(v),
            AnyValueDef::UInt64(v) => UInt64(v),
            AnyValueDef::UInt128(v) => UInt128(v),
            AnyValueDef::Int8(v) => Int8(v),
            AnyValueDef::Int16(v) => Int16(v),
            AnyValueDef::Int32(v) => Int32(v),
            AnyValueDef::Int64(v) => Int64(v),
            AnyValueDef::Int128(v) => Int128(v),
            AnyValueDef::Float32(v) => Float32(v),
            AnyValueDef::Float64(v) => Float64(v),
            AnyValueDef::Char(v) => Char(v),
            AnyValueDef::Str(v) => StrOwned(v), // Deserialize as owned string
            AnyValueDef::StrOwned(v) => StrOwned(v), // Deserialize as owned string
            AnyValueDef::Duration(ms) => Duration(std::time::Duration::from_millis(ms)),
            AnyValueDef::Slice(vals) => Vector(vals.into_iter().map(|v| v.into()).collect()),
            AnyValueDef::Vector(vals) => Vector(vals.into_iter().map(|v| v.into()).collect()),
            AnyValueDef::Struct(vals) => {
                Struct(vals.into_iter().map(|(f, v)| (f, v.into())).collect())
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{DataType, Field};

    use super::AnyValue;

    #[test]
    fn test_anyvalue_equality() {
        let v1 = AnyValue::Struct(vec![
            (
                Field::from(("field1", DataType::Int32)),
                AnyValue::Int32(42),
            ),
            (
                Field::from(("field2", DataType::String)),
                AnyValue::Str("hello"),
            ),
        ]);

        let v2 = AnyValue::Struct(vec![
            (
                Field::from(("field1", DataType::Int32)),
                AnyValue::Int32(42),
            ),
            (
                Field::from(("field2", DataType::String)),
                AnyValue::Str("hello"),
            ),
        ]);

        let v3 = AnyValue::Struct(vec![
            (
                Field::from(("field1", DataType::Int32)),
                AnyValue::Int32(43),
            ),
            (
                Field::from(("field2", DataType::String)),
                AnyValue::Str("hello"),
            ),
        ]);

        assert_eq!(v1, v2);
        assert_ne!(v1, v3);
    }

    #[test]
    fn test_anyvalue_type_name() {
        let v = AnyValue::Float64(3.14);
        assert_eq!(v.type_name(), "f64");
    }
}
