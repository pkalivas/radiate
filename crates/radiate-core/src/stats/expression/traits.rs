use radiate_error::RadiateError;
use radiate_utils::AnyValue;

pub(crate) type ExprResult<'a> = Result<AnyValue<'a>, RadiateError>;

pub trait Evaluate<I> {
    fn eval<'a>(&'a mut self, input: &I) -> ExprResult<'a>;
}
