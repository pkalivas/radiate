use crate::{AnyValue, Expr, ExprProjection, ExprQuery, ExprResult, Field};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub enum PathSegment {
    Key(AnyValue<'static>),
    Index(usize),
    StructField(Field),
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct PathBuilder {
    path: Vec<PathSegment>,
}

impl PathBuilder {
    pub fn key(mut self, key: impl Into<AnyValue<'static>>) -> Self {
        self.path.push(PathSegment::Key(key.into()));
        self
    }

    pub fn index(mut self, index: usize) -> Self {
        self.path.push(PathSegment::Index(index));
        self
    }

    pub fn field(mut self, field: Field) -> Self {
        self.path.push(PathSegment::StructField(field));
        self
    }
}

impl Into<Expr> for PathBuilder {
    fn into(self) -> Expr {
        Expr::Selector(SelectExpr::Path(self.path))
    }
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum SelectExpr {
    Field(AnyValue<'static>, Field),
    Nth(usize),
    Path(Vec<PathSegment>),
}

impl<T> ExprQuery<T> for SelectExpr
where
    T: ExprProjection,
{
    fn dispatch<'a>(&'a mut self, input: &T) -> ExprResult<'a> {
        Ok(input.project(self).unwrap_or(AnyValue::Null))
    }
}
