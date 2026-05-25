use super::Expr;
use super::ops::{TrinaryExpr, TrinaryOp};

#[derive(Clone, Debug, PartialEq)]
pub struct When {
    pub(super) cond: Expr,
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
    pub(super) cond: Expr,
    pub(super) then_expr: Expr,
}

impl Then {
    pub fn otherwise(self, else_expr: impl Into<Expr>) -> Expr {
        Expr::Trinary(TrinaryExpr::new(
            self.cond,
            self.then_expr,
            else_expr.into(),
            TrinaryOp::If,
        ))
    }
}
