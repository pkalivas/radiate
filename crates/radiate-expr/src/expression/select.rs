use crate::{AnyValue, Expr, ExprProjection, ExprQuery, ExprResult, Field};
use radiate_error::radiate_bail;
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

impl From<PathBuilder> for Expr {
    fn from(val: PathBuilder) -> Self {
        Expr::Selector(SelectExpr::Path(val.path))
    }
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum SelectExpr {
    Field(AnyValue<'static>, Field),
    Nth(usize),
    Path(Vec<PathSegment>),
    Element,
}

impl<T> ExprQuery<T> for SelectExpr
where
    T: ExprProjection,
{
    fn dispatch<'a>(&'a mut self, input: &T) -> ExprResult<'a> {
        if let Some(result) = input.project(self) {
            Ok(result)
        } else {
            radiate_bail!(Expr: "Failed to project value using selector {:?}", self)
        }
    }
}
