use crate::{AnyValue, Expr, ExprProjection, ExprQuery};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UnaryOp {
    Not,
    Neg,
    Abs,
}

#[derive(Clone, Debug, PartialEq)]
pub struct UnaryExpr {
    pub(super) child: Box<Expr>,
    pub(super) op: UnaryOp,
}

impl UnaryExpr {
    pub fn new(child: Expr, op: UnaryOp) -> Self {
        Self {
            child: Box::new(child),
            op,
        }
    }
}

impl<T> ExprQuery<T> for UnaryExpr
where
    T: ExprProjection,
{
    fn dispatch<'a>(&'a mut self, input: &T) -> AnyValue<'a> {
        let value = self.child.dispatch(input);

        match self.op {
            UnaryOp::Not => match value {
                AnyValue::Bool(b) => AnyValue::Bool(!b),
                _ => AnyValue::Null,
            },
            UnaryOp::Neg => match value.extract::<f32>() {
                Some(v) => AnyValue::Float32(-v),
                None => AnyValue::Null,
            },
            UnaryOp::Abs => match value.extract::<f32>() {
                Some(v) => AnyValue::Float32(v.abs()),
                None => AnyValue::Null,
            },
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    And,
    Or,
    Lt,
    Lte,
    Gt,
    Gte,
    Eq,
    Ne,
    Mod,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BinaryExpr {
    pub(super) lhs: Box<Expr>,
    pub(super) rhs: Box<Expr>,
    pub(super) op: BinaryOp,
}

impl BinaryExpr {
    pub fn new(lhs: Expr, rhs: Expr, op: BinaryOp) -> Self {
        Self {
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
            op,
        }
    }
}

impl<T> ExprQuery<T> for BinaryExpr
where
    T: ExprProjection,
{
    fn dispatch<'a>(&'a mut self, input: &T) -> AnyValue<'a> {
        let lhs = self.lhs.dispatch(input);
        let rhs = self.rhs.dispatch(input);

        match self.op {
            BinaryOp::Add => lhs + rhs,
            BinaryOp::Sub => lhs - rhs,
            BinaryOp::Mul => lhs * rhs,
            BinaryOp::Div => lhs / rhs,
            BinaryOp::Lt => AnyValue::Bool(lhs < rhs),
            BinaryOp::Lte => AnyValue::Bool(lhs <= rhs),
            BinaryOp::Gt => AnyValue::Bool(lhs > rhs),
            BinaryOp::Gte => AnyValue::Bool(lhs >= rhs),
            BinaryOp::Eq => AnyValue::Bool(lhs == rhs),
            BinaryOp::Ne => AnyValue::Bool(lhs != rhs),
            BinaryOp::And => lhs & rhs,
            BinaryOp::Or => lhs | rhs,
            BinaryOp::Mod => lhs % rhs,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TrinaryOp {
    If,
    Clamp,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TrinaryExpr {
    pub(super) first: Box<Expr>,
    pub(super) second: Box<Expr>,
    pub(super) third: Box<Expr>,
    pub(super) operation: TrinaryOp,
}

impl TrinaryExpr {
    pub fn new(first: Expr, second: Expr, third: Expr, operation: TrinaryOp) -> Self {
        Self {
            first: Box::new(first),
            second: Box::new(second),
            third: Box::new(third),
            operation,
        }
    }
}

impl<T> ExprQuery<T> for TrinaryExpr
where
    T: ExprProjection,
{
    fn dispatch<'a>(&'a mut self, input: &T) -> AnyValue<'a> {
        match self.operation {
            TrinaryOp::If => {
                let condition = self.first.dispatch(input);

                let cond = match condition {
                    AnyValue::Bool(b) => b,
                    _ => return AnyValue::Null,
                };

                if cond {
                    self.second.dispatch(input)
                } else {
                    self.third.dispatch(input)
                }
            }
            TrinaryOp::Clamp => {
                let value = self.first.dispatch(input).extract::<f32>();
                let min = self.second.dispatch(input).extract::<f32>();
                let max = self.third.dispatch(input).extract::<f32>();

                match (value, min, max) {
                    (Some(value), Some(min), Some(max)) => AnyValue::Float32(value.clamp(min, max)),
                    _ => AnyValue::Null,
                }
            }
        }
    }
}
