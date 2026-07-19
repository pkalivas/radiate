use radiate_error::RadiateError;

use crate::AnyValue;

impl<'a> TryFrom<AnyValue<'a>> for f32 {
    type Error = RadiateError;

    fn try_from(value: AnyValue<'a>) -> Result<Self, Self::Error> {
        match value {
            AnyValue::Float32(v) => Ok(v),
            AnyValue::Float64(v) => Ok(v as f32),
            AnyValue::Int8(v) => Ok(v as f32),
            AnyValue::Int16(v) => Ok(v as f32),
            AnyValue::Int32(v) => Ok(v as f32),
            AnyValue::Int64(v) => Ok(v as f32),
            AnyValue::UInt8(v) => Ok(v as f32),
            AnyValue::UInt16(v) => Ok(v as f32),
            AnyValue::UInt32(v) => Ok(v as f32),
            AnyValue::UInt64(v) => Ok(v as f32),
            AnyValue::Bool(v) => Ok(if v { 1.0 } else { 0.0 }),
            AnyValue::Usize(v) => Ok(v as f32),
            AnyValue::Duration(v) => Ok(v.as_secs_f32()),
            _ => Err(RadiateError::AnyValue(format!(
                "Expected Float32, found {:?}",
                value.dtype()
            ))),
        }
    }
}
