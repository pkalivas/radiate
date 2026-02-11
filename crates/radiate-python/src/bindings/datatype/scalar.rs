use super::{AnyValue, DataType};

#[derive(Debug, Clone, PartialEq)]
pub struct Scalar {
    dtype: DataType,
    value: AnyValue<'static>,
}

impl Scalar {
    pub fn new(dtype: DataType, value: AnyValue<'static>) -> Self {
        Scalar { dtype, value }
    }

    pub fn dtype(&self) -> &DataType {
        &self.dtype
    }

    pub fn value(&self) -> &AnyValue<'static> {
        &self.value
    }

    pub fn into_value(self) -> AnyValue<'static> {
        self.value
    }

    pub fn extract<T>(self) -> Option<T>
    where
        T: num_traits::NumCast,
    {
        self.value.extract()
    }
}

macro_rules! impl_from {
    ($(($t:ty, $av:ident, $dt:ident))+) => {
        $(
            impl From<$t> for Scalar {
                #[inline]
                fn from(v: $t) -> Self {
                    Self::new(DataType::$dt, AnyValue::$av(v))
                }
            }
        )+
    }
}

impl_from! {
    (bool, Bool, Boolean)
    (i8, Int8, Int8)
    (i16, Int16, Int16)
    (i32, Int32, Int32)
    (i64, Int64, Int64)
    (i128, Int128, Int128)
    (u8, UInt8, UInt8)
    (u16, UInt16, UInt16)
    (u32, UInt32, UInt32)
    (u64, UInt64, UInt64)
    (u128, UInt128, UInt128)
    (f32, Float32, Float32)
    (f64, Float64, Float64)
    (String, StrOwned, String)
}
