use crate::MetricSet;
use radiate_error::RadiateError;
use radiate_utils::AnyValue;

pub(crate) type ExprResult<'a> = Result<AnyValue<'a>, RadiateError>;

pub trait Evaluate {
    fn eval<'a>(&'a mut self, metrics: &MetricSet) -> ExprResult<'a>;
}
