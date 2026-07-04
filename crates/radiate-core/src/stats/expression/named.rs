use crate::{
    Evaluate,
    stats::{ExprResult, ExprSelector},
};

use super::Expr;
use radiate_utils::SmallStr;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize, ser::SerializeStruct};

#[derive(Clone, Debug, PartialEq)]
pub struct NamedExpr {
    pub name: SmallStr,
    pub expr: Expr,
}

impl NamedExpr {
    pub fn new(name: impl Into<SmallStr>, expr: Expr) -> Self {
        Self {
            name: name.into(),
            expr: expr.compile(),
        }
    }

    pub fn pair(&mut self) -> (&str, &mut Expr) {
        (self.name.as_str(), &mut self.expr)
    }

    pub fn expr(&self) -> &Expr {
        &self.expr
    }

    pub fn expr_mut(&mut self) -> &mut Expr {
        &mut self.expr
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }
}

impl<T> Evaluate<T> for NamedExpr
where
    T: ExprSelector,
{
    fn eval<'a>(&'a mut self, metrics: &T) -> ExprResult<'a> {
        match &mut self.expr {
            Expr::Literal(value) => Ok(value.clone()),
            Expr::Selector(selector) => selector.eval(metrics),
            Expr::Aggregate(child) => child.eval(metrics),
            Expr::Trinary(child) => child.eval(metrics),
            Expr::Binary(child) => child.eval(metrics),
            Expr::Unary(child) => child.eval(metrics),
            Expr::Schedule(child) => child.eval(metrics),
        }
    }
}

impl<T: Into<SmallStr>> From<(T, Expr)> for NamedExpr {
    fn from((name, expr): (T, Expr)) -> Self {
        Self::new(name, expr)
    }
}

#[cfg(feature = "serde")]
impl Serialize for NamedExpr {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("MetricQuery", 2)?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("expr", &self.expr)?;
        state.end()
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for NamedExpr {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct NamedExprData {
            name: String,
            expr: Expr,
        }

        let data = NamedExprData::deserialize(deserializer)?;
        Ok(NamedExpr::new(radiate_utils::intern!(data.name), data.expr))
    }
}
