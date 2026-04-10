use crate::{AnyValue, Expr, ExprProjection, ExprQuery, Field};

/// Examples of SelectExpr:
///
/// // accuracy.last_value
/// SelectExpr::Path(vec![
///     PathSegment::Key(AnyValue::from("accuracy").into_static()),
///     PathSegment::StructField(expr_fields::LAST_VALUE.clone()),
/// ])
///
/// // users[0].name
/// SelectExpr::Path(vec![
///     PathSegment::Key(AnyValue::from("users").into_static()),
///     PathSegment::Index(0),
///     PathSegment::Key(AnyValue::from("name").into_static()),
/// ])
///
/// // portfolio.positions[3].delta
/// SelectExpr::Path(vec![
///     PathSegment::Key(AnyValue::from("portfolio").into_static()),
///     PathSegment::Key(AnyValue::from("positions").into_static()),
///     PathSegment::Index(3),
///     PathSegment::Key(AnyValue::from("delta").into_static()),
/// ])
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
pub enum SelectExpr {
    Field(AnyValue<'static>, Field),
    Nth(usize),
    Path(Vec<PathSegment>),
}

impl<T> ExprQuery<T> for SelectExpr
where
    T: ExprProjection,
{
    fn dispatch<'a>(&'a mut self, input: &T) -> AnyValue<'a> {
        input.project(self).unwrap_or(AnyValue::Null)
    }
}
