use crate::Expr;
use radiate_error::RadiateError;
use radiate_utils::SmallStr;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ExprSet {
    pub exprs: Vec<Expr>,
}

impl ExprSet {
    pub fn new(exprs: Vec<Expr>) -> Self {
        Self { exprs }
    }

    pub fn get(&self, name: impl AsRef<str>) -> Option<&Expr> {
        let name = name.as_ref();
        self.exprs.iter().find(|e| e.name() == name)
    }

    pub fn len(&self) -> usize {
        self.exprs.len()
    }

    pub fn add(&mut self, expr: impl Into<Expr>) {
        self.exprs.push(expr.into());
    }

    pub fn iter(&self) -> impl Iterator<Item = &Expr> {
        self.exprs.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Expr> {
        self.exprs.iter_mut()
    }

    pub fn apply_mut<F>(&mut self, mut f: F) -> Result<(), RadiateError>
    where
        F: FnMut(&mut Expr) -> Result<(), RadiateError>,
    {
        for expr in &mut self.exprs {
            f(expr)?;
        }
        Ok(())
    }
}

impl From<Expr> for ExprSet {
    fn from(expr: Expr) -> Self {
        Self::new(vec![expr])
    }
}

impl<const N: usize> From<[Expr; N]> for ExprSet {
    fn from(exprs: [Expr; N]) -> Self {
        Self::new(exprs.into_iter().collect())
    }
}

impl From<Vec<Expr>> for ExprSet {
    fn from(exprs: Vec<Expr>) -> Self {
        Self::new(exprs)
    }
}

impl From<Vec<(SmallStr, Expr)>> for ExprSet {
    fn from(exprs: Vec<(SmallStr, Expr)>) -> Self {
        let named_exprs = exprs
            .into_iter()
            .map(|(name, expr)| expr.alias(name))
            .collect();
        Self::new(named_exprs)
    }
}
