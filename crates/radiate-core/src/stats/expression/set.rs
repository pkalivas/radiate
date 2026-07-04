use radiate_utils::{AnyValue, SmallStr};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{
    Evaluate, Expr, NamedExpr,
    stats::{ExprResult, ExprSelector},
};

#[derive(Clone, Debug, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ExprSet {
    pub exprs: Vec<NamedExpr>,
}

impl ExprSet {
    pub fn new(exprs: Vec<NamedExpr>) -> Self {
        Self { exprs }
    }

    pub fn iter(&self) -> impl Iterator<Item = &NamedExpr> {
        self.exprs.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut NamedExpr> {
        self.exprs.iter_mut()
    }
}

impl<T> Evaluate<T> for ExprSet
where
    T: ExprSelector,
{
    fn eval<'a>(&'a mut self, metrics: &T) -> ExprResult<'a> {
        let mut results = Vec::with_capacity(self.exprs.len());
        for expr in &mut self.exprs {
            let value = expr.eval(metrics)?;
            results.push(value);
        }

        Ok(AnyValue::Vector(results))
    }
}

impl From<NamedExpr> for ExprSet {
    fn from(expr: NamedExpr) -> Self {
        Self::new(vec![expr])
    }
}

impl<const N: usize> From<[NamedExpr; N]> for ExprSet {
    fn from(exprs: [NamedExpr; N]) -> Self {
        Self::new(exprs.into())
    }
}

impl From<Vec<NamedExpr>> for ExprSet {
    fn from(exprs: Vec<NamedExpr>) -> Self {
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
