use crate::AnyValue;
use radiate_error::RadiateError;

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

impl<'a> TryFrom<AnyValue<'a>> for String {
    type Error = RadiateError;

    fn try_from(value: AnyValue<'a>) -> Result<Self, Self::Error> {
        match value {
            AnyValue::StrOwned(v) => Ok(v.to_string()),
            AnyValue::Str(v) => Ok(v.to_string()),
            _ => Err(RadiateError::AnyValue(format!(
                "Expected String, found {:?}",
                value.dtype()
            ))),
        }
    }
}

impl<'a, T> TryFrom<AnyValue<'a>> for Vec<T>
where
    T: TryFrom<AnyValue<'a>, Error = RadiateError>,
{
    type Error = RadiateError;

    fn try_from(value: AnyValue<'a>) -> Result<Self, Self::Error> {
        match value {
            AnyValue::Vector(v) => v
                .into_iter()
                .map(|v| T::try_from(v))
                .collect::<Result<Vec<T>, RadiateError>>(),
            AnyValue::Slice(slice) => slice
                .iter()
                .map(|v| T::try_from(v.clone()))
                .collect::<Result<Vec<T>, RadiateError>>(),
            _ => Err(RadiateError::AnyValue(format!(
                "Expected Vector, found {:?}",
                value.dtype()
            ))),
        }
    }
}
