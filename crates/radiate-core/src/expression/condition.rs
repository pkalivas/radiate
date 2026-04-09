use crate::ExprNode;

pub struct When<'a> {
    cond: ExprNode<'a>,
}

impl<'a> When<'a> {
    pub fn new(cond: impl Into<ExprNode<'a>>) -> Self {
        Self { cond: cond.into() }
    }

    pub fn then(self, then_expr: impl Into<ExprNode<'a>>) -> Then<'a> {
        Then {
            cond: self.cond,
            then_expr: then_expr.into(),
        }
    }
}

pub struct Then<'a> {
    cond: ExprNode<'a>,
    then_expr: ExprNode<'a>,
}

impl<'a> Then<'a> {
    pub fn otherwise(self, else_expr: impl Into<ExprNode<'a>>) -> ExprNode<'a> {
        ExprNode::If {
            cond: Box::new(self.cond),
            then_expr: Box::new(self.then_expr),
            else_expr: Box::new(else_expr.into()),
        }
    }
}
