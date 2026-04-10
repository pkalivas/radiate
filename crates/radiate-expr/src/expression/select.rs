use crate::{AnyValue, ExprProjection, ExprQuery, Field};

#[derive(Clone, Debug, PartialEq)]
pub enum SelectExpr {
    Field(AnyValue<'static>, Field),
    Nth(usize),
}

impl<T> ExprQuery<T> for SelectExpr
where
    T: ExprProjection,
{
    fn dispatch<'a>(&'a mut self, input: &T) -> AnyValue<'a> {
        match self {
            SelectExpr::Field(value, field) => {
                input.project(value, field).unwrap_or(AnyValue::Null)
            }
            _ => AnyValue::Null, // TODO: implement Nth
        }
    }
}
