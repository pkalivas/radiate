use radiate_error::RadiateError;
use radiate_utils::AnyValue;
use std::time::Duration;

use crate::{Expr, nodes::Selector};

pub(crate) type ExprResult<'a, O = AnyValue<'a>> = Result<O, RadiateError>;

/// A selector for expression trees that need no metric input — schedules,
/// pure literals, throttles. `select` should never actually be reached by a
/// well-formed schedule tree; it exists to satisfy `T: ExprSelect<'a>`.
pub struct NoInput;

impl<'a> ExprSelect<'a> for NoInput {
    fn select(&'a self, _sel: &Selector) -> AnyValue<'a> {
        AnyValue::Null
    }
}

pub trait EvalNoInput: Sized {
    fn evaluate(&mut self) -> ExprResult<'static>;
}

impl EvalNoInput for Expr {
    fn evaluate(&mut self) -> ExprResult<'static> {
        EvalExpr::evaluate(self, &NoInput).map(AnyValue::into_static)
    }
}

pub trait EvalExpr<'a, I: ExprSelect<'a>, O = AnyValue<'a>> {
    fn evaluate(&'a mut self, input: &'a I) -> ExprResult<'a, O>;
}

pub trait ExprSelect<'a> {
    fn select(&'a self, expr: &Selector) -> AnyValue<'a>;
}

impl<'a, T: ExprSelect<'a>> ExprSelect<'a> for Vec<T> {
    fn select(&'a self, expr: &Selector) -> AnyValue<'a> {
        match expr {
            Selector::Identity => AnyValue::Vector(
                self.iter()
                    .map(|v| v.select(&Selector::Identity))
                    .collect::<Vec<AnyValue<'a>>>(),
            ),
            Selector::Index(idx) => self
                .get(*idx)
                .map(|v| v.select(&Selector::Identity))
                .unwrap_or(AnyValue::Null),
            Selector::Range(start, end) => {
                let slice = self.get(*start..*end).unwrap_or(&[]);
                let values = slice
                    .iter()
                    .map(|v| v.select(&Selector::Identity))
                    .collect::<Vec<AnyValue<'a>>>();
                AnyValue::Vector(values)
            }
            _ => AnyValue::Null,
        }
    }
}

impl<'a> ExprSelect<'a> for AnyValue<'a> {
    fn select(&'a self, _: &Selector) -> AnyValue<'a> {
        self.clone()
    }
}

impl<'a> ExprSelect<'a> for String {
    fn select(&'a self, _: &Selector) -> AnyValue<'a> {
        AnyValue::Str(self)
    }
}

macro_rules! impl_select {
    ($t:ty, $dtype:ident) => {
        impl<'a> ExprSelect<'a> for $t {
            fn select(&'a self, _: &Selector) -> AnyValue<'a> {
                AnyValue::$dtype(*self)
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
