use crate::{Evaluate, Expr, ExprResult, ExprSelector};
use radiate_utils::SmallStr;
use radiate_utils::sentry_id;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize, ser::SerializeStruct};
use std::sync::atomic::AtomicU64;

sentry_id!(ExprId);

#[derive(Clone, Debug, PartialEq)]
pub struct NamedExpr {
    pub id: ExprId,
    pub name: SmallStr,
    pub expr: Expr,
}

impl NamedExpr {
    pub fn new(name: impl Into<SmallStr>, expr: Expr) -> Self {
        Self {
            id: ExprId::new(),
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

    pub fn name(&self) -> &SmallStr {
        &self.name
    }
}

impl<'a, T> Evaluate<'a, T> for NamedExpr
where
    T: ExprSelector,
{
    fn eval(&'a mut self, metrics: &T) -> ExprResult<'a> {
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

impl From<Expr> for NamedExpr {
    fn from(expr: Expr) -> Self {
        let id = ExprId::new();
        NamedExpr {
            id,
            name: SmallStr::from_string(format!("Named.{:?}", id)),
            expr,
        }
    }
}

impl<T: Into<SmallStr>> From<(T, Expr)> for NamedExpr {
    fn from((name, expr): (T, Expr)) -> Self {
        Self::new(name, expr)
    }
}

impl From<f32> for NamedExpr {
    fn from(value: f32) -> Self {
        Self::from(Expr::lit(value))
    }
}

impl From<f64> for NamedExpr {
    fn from(value: f64) -> Self {
        Self::from(Expr::lit(value))
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
