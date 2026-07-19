use super::ops::{TrinaryExpr, TrinaryOp};
use crate::{Expr, expr::ExprKind};

#[derive(Clone, Debug, PartialEq)]
pub struct When {
    pub(crate) cond: Expr,
}

impl When {
    pub fn new(cond: impl Into<Expr>) -> Self {
        Self { cond: cond.into() }
    }

    pub fn then(self, then_expr: impl Into<Expr>) -> Then {
        Then {
            cond: self.cond,
            then_expr: then_expr.into(),
        }
    }
}

pub struct Then {
    pub(crate) cond: Expr,
    pub(crate) then_expr: Expr,
}

impl Then {
    pub fn otherwise(self, else_expr: impl Into<Expr>) -> Expr {
        Expr::new(ExprKind::Trinary(TrinaryExpr::new(
            self.cond,
            self.then_expr,
            else_expr.into(),
            TrinaryOp::If,
        )))
    }
}
