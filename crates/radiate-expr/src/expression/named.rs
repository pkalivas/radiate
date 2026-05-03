use crate::Expr;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize, ser::SerializeStruct};

#[derive(Clone, Debug, PartialEq)]
pub struct NamedExpr {
    pub name: &'static str,
    pub expr: Expr,
}

impl NamedExpr {
    pub fn new(name: &'static str, expr: Expr) -> Self {
        Self { name, expr }
    }

    pub fn pair(&mut self) -> (&'static str, &mut Expr) {
        (self.name, &mut self.expr)
    }

    pub fn expr(&self) -> &Expr {
        &self.expr
    }

    pub fn expr_mut(&mut self) -> &mut Expr {
        &mut self.expr
    }

    pub fn name(&self) -> &'static str {
        self.name
    }
}

impl From<(&'static str, Expr)> for NamedExpr {
    fn from((name, expr): (&'static str, Expr)) -> Self {
        Self::new(name, expr)
    }
}

#[cfg(feature = "serde")]
impl Serialize for NamedExpr {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("NamedExpr", 2)?;
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
