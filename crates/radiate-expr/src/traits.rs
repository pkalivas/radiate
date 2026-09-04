use crate::{Expr, nodes::Selector};
use radiate_error::RadiateError;
use radiate_utils::AnyValue;
use std::time::Duration;

pub(crate) type ExprResult<'a, O = AnyValue<'a>> = Result<O, RadiateError>;

struct NoInput;
impl<'a> ExprSelect<'a> for NoInput {
    fn select(&'a self, _sel: &Selector) -> Result<AnyValue<'a>, RadiateError> {
        Ok(AnyValue::Null)
    }
}

pub trait EvalNoInput: Sized {
    fn compute(&mut self) -> ExprResult<'static>;
}

impl EvalNoInput for Expr {
    fn compute(&mut self) -> ExprResult<'static> {
        EvalExpr::evaluate(self, &NoInput).map(AnyValue::into_static)
    }
}

pub trait EvalExpr<'a, I: ExprSelect<'a>, O = AnyValue<'a>> {
    fn evaluate(&'a mut self, input: &'a I) -> ExprResult<'a, O>;
}

pub trait ExprSelect<'a> {
    fn select(&'a self, expr: &Selector) -> Result<AnyValue<'a>, RadiateError>;
}

impl<'a, T: ExprSelect<'a>> ExprSelect<'a> for Vec<T> {
    fn select(&'a self, expr: &Selector) -> Result<AnyValue<'a>, RadiateError> {
        match expr {
            Selector::Identity => Ok(AnyValue::Vector(
                self.iter()
                    .filter_map(|v| v.select(&Selector::Identity).ok())
                    .collect::<Vec<AnyValue<'a>>>(),
            )),
            Selector::Index(idx) => self
                .get(*idx)
                .map(|v| v.select(&Selector::Identity))
                .unwrap_or(Ok(AnyValue::Null)),
            Selector::Range(start, end) => {
                let slice = self.get(*start..*end).unwrap_or(&[]);
                let values = slice
                    .iter()
                    .filter_map(|v| v.select(&Selector::Identity).ok())
                    .collect::<Vec<AnyValue<'a>>>();
                Ok(AnyValue::Vector(values))
            }
            _ => Ok(AnyValue::Null),
        }
    }
}

impl<'a> ExprSelect<'a> for AnyValue<'a> {
    fn select(&'a self, _: &Selector) -> Result<AnyValue<'a>, RadiateError> {
        Ok(self.clone())
    }
}

impl<'a> ExprSelect<'a> for String {
    fn select(&'a self, _: &Selector) -> Result<AnyValue<'a>, RadiateError> {
        Ok(AnyValue::Str(self))
    }
}

macro_rules! impl_select {
    ($t:ty, $dtype:ident) => {
        impl<'a> ExprSelect<'a> for $t {
            fn select(&'a self, _: &Selector) -> Result<AnyValue<'a>, RadiateError> {
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
