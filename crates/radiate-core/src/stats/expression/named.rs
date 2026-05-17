use super::Expr;
use radiate_utils::SmallStr;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize, ser::SerializeStruct};

#[derive(Clone, Debug, PartialEq)]
pub struct MetricQuery {
    pub name: SmallStr,
    pub expr: Expr,
}

impl MetricQuery {
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

impl<T: Into<SmallStr>> From<(T, Expr)> for MetricQuery {
    fn from((name, expr): (T, Expr)) -> Self {
        Self::new(name, expr)
    }
}

#[cfg(feature = "serde")]
impl Serialize for MetricQuery {
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
impl<'de> Deserialize<'de> for MetricQuery {
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
        Ok(MetricQuery::new(
            radiate_utils::intern!(data.name),
            data.expr,
        ))
    }
}
