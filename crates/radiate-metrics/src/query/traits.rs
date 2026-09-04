use crate::Selector;
use radiate_error::RadiateError;
use radiate_utils::AnyValue;

pub(crate) type ExprResult<'a, O = AnyValue<'a>> = Result<O, RadiateError>;

pub trait ExprEval<'a, I, O = AnyValue<'a>>
where
    I: ExprSelect,
{
    fn evaluate(&'a mut self, input: &I) -> ExprResult<'a, O>;
}

pub trait ExprSelect {
    fn select(&self, expr: &Selector) -> AnyValue<'static>;
}

impl ExprSelect for () {
    fn select(&self, _expr: &Selector) -> AnyValue<'static> {
        AnyValue::Null
    }
}

impl<T> ExprSelect for Vec<T>
where
    T: ExprSelect,
{
    fn select(&self, expr: &Selector) -> AnyValue<'static> {
        match expr {
            Selector::Identity => AnyValue::Null,
            Selector::Index(idx) => self
                .get(*idx)
                .map(|v| v.select(&Selector::Identity))
                .unwrap_or(AnyValue::Null),
            Selector::Range(start, end) => {
                let slice = self.get(*start..*end).unwrap_or(&[]);
                let values = slice
                    .iter()
                    .map(|v| v.select(&Selector::Identity))
                    .collect::<Vec<AnyValue<'static>>>();
                AnyValue::Vector(values)
            }
            _ => AnyValue::Null,
        }
    }
}

macro_rules! impl_select {
    ($t:ty, $dtype:ident) => {
        impl ExprSelect for $t {
            fn select(&self, _expr: &Selector) -> AnyValue<'static> {
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
