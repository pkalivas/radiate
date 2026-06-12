use crate::SelectExpr;
use radiate_error::RadiateError;
use radiate_utils::AnyValue;

pub(crate) type ExprResult<'a> = Result<AnyValue<'a>, RadiateError>;

pub trait Evaluate<T>
where
    T: ExprSelector,
{
    fn eval<'a>(&'a mut self, metrics: &T) -> ExprResult<'a>;
}

pub trait ExprSelector {
    fn select(&self, expr: &SelectExpr) -> AnyValue<'static>;
}

impl ExprSelector for () {
    fn select(&self, _expr: &SelectExpr) -> AnyValue<'static> {
        AnyValue::Null
    }
}

macro_rules! impl_select {
    ($t:ty, $dtype:ident) => {
        impl ExprSelector for $t {
            fn select(&self, _expr: &SelectExpr) -> AnyValue<'static> {
                AnyValue::$dtype(*self)
            }
        }
    };
}

impl_select!(u8, UInt8);
impl_select!(u16, UInt16);
impl_select!(u32, UInt32);
impl_select!(u64, UInt64);

impl_select!(i8, Int8);
impl_select!(i16, Int16);
impl_select!(i32, Int32);
impl_select!(i64, Int64);

impl_select!(bool, Bool);

impl_select!(f32, Float32);
impl_select!(f64, Float64);
