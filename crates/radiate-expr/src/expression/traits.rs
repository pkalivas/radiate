use super::Expr;
use crate::AnyValue;
use radiate_error::RadiateError;

pub(crate) type ExprResult<'a> = Result<AnyValue<'a>, RadiateError>;

pub trait ApplyExpr<'a> {
    fn apply(&self, expr: &'a mut Expr) -> AnyValue<'a>;
}

pub trait ExprQuery<I> {
    fn dispatch<'a>(&'a mut self, input: &I) -> ExprResult<'a>;
}
