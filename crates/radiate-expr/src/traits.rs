use crate::ops::SelectOp;
use radiate_error::RadiateError;
use radiate_utils::AnyValue;
use std::time::Duration;

pub(crate) type ExprResult<'a, O = AnyValue<'a>> = Result<O, RadiateError>;

pub trait ProjectExpr<'a> {
    fn select(&'a self, expr: &SelectOp) -> Result<AnyValue<'a>, RadiateError>;
}

impl<'a, T, F> ProjectExpr<'a> for F
where
    T: Into<AnyValue<'a>>,
    F: Fn(&SelectOp) -> Result<T, RadiateError>,
{
    fn select(&'a self, expr: &SelectOp) -> Result<AnyValue<'a>, RadiateError> {
        self(expr).map(|v| v.into())
    }
}

impl<'a, T: ProjectExpr<'a>> ProjectExpr<'a> for Vec<T> {
    fn select(&'a self, expr: &SelectOp) -> Result<AnyValue<'a>, RadiateError> {
        match expr {
            SelectOp::Identity => Ok(AnyValue::Vector(
                self.iter()
                    .filter_map(|v| v.select(&SelectOp::Identity).ok())
                    .collect::<Vec<AnyValue<'a>>>(),
            )),
            SelectOp::Index(idx) => self
                .get(*idx)
                .map(|v| v.select(&SelectOp::Identity))
                .unwrap_or(Ok(AnyValue::Null)),
            SelectOp::Range(start, end) => {
                let slice = self.get(*start..*end).unwrap_or(&[]);
                let values = slice
                    .iter()
                    .filter_map(|v| v.select(&SelectOp::Identity).ok())
                    .collect::<Vec<AnyValue<'a>>>();
                Ok(AnyValue::Vector(values))
            }
            _ => Ok(AnyValue::Null),
        }
    }
}

impl<'a> ProjectExpr<'a> for AnyValue<'a> {
    fn select(&'a self, _: &SelectOp) -> Result<AnyValue<'a>, RadiateError> {
        Ok(self.clone())
    }
}

impl<'a> ProjectExpr<'a> for String {
    fn select(&'a self, _: &SelectOp) -> Result<AnyValue<'a>, RadiateError> {
        Ok(AnyValue::Str(self))
    }
}

macro_rules! impl_select {
    ($t:ty, $dtype:ident) => {
        impl<'a> ProjectExpr<'a> for $t {
            fn select(&'a self, _: &SelectOp) -> Result<AnyValue<'a>, RadiateError> {
                Ok(AnyValue::$dtype(*self))
            }
        }
    };
}

impl_select!(u8, UInt8);
impl_select!(u16, UInt16);
impl_select!(u32, UInt32);
impl_select!(u64, UInt64);
impl_select!(u128, UInt128);

impl_select!(i8, Int8);
impl_select!(i16, Int16);
impl_select!(i32, Int32);
impl_select!(i64, Int64);
impl_select!(i128, Int128);

impl_select!(f32, Float32);
impl_select!(f64, Float64);

impl_select!(Duration, Duration);
impl_select!(bool, Bool);
impl_select!(usize, Usize);

impl_select!(char, Char);
impl_select!(&str, Str);
