use crate::Expr;
use radiate_utils::SmallStr;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ExprSet {
    pub exprs: HashMap<SmallStr, Expr>,
}

impl ExprSet {
    pub fn new(exprs: Vec<Expr>) -> Self {
        Self {
            exprs: exprs.into_iter().map(|e| (e.name().into(), e)).collect(),
        }
    }

    pub fn get(&self, name: impl AsRef<str>) -> Option<&Expr> {
        let name = name.as_ref();
        self.exprs.get(name)
    }

    pub fn len(&self) -> usize {
        self.exprs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.exprs.is_empty()
    }

    pub fn push(&mut self, expr: impl Into<Expr>) {
        let expr = expr.into();
        self.exprs.insert(expr.name().into(), expr);
    }

    pub fn add(&mut self, name: impl Into<SmallStr>, expr: impl Into<Expr>) {
        let expr = expr.into();
        self.exprs.insert(name.into(), expr);
    }

    pub fn iter(&self) -> impl Iterator<Item = (&SmallStr, &Expr)> {
        self.exprs.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&SmallStr, &mut Expr)> {
        self.exprs.iter_mut()
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
