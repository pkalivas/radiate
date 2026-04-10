use crate::{AnyValue, Expr, ExprProjection, ExprQuery};
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UnaryOp {
    Not,
    Neg,
    Abs,
}

#[derive(Clone, Debug, PartialEq)]
pub struct UnaryExpr {
    pub(super) child: Arc<Expr>,
    pub(super) op: UnaryOp,
}

impl UnaryExpr {
    pub fn new(child: Expr, op: UnaryOp) -> Self {
        Self {
            child: Arc::new(child),
            op,
        }
    }
}

impl<T> ExprQuery<T> for UnaryExpr
where
    T: ExprProjection,
{
    fn dispatch<'a>(&'a mut self, input: &T) -> AnyValue<'a> {
        let value = Arc::make_mut(&mut self.child).dispatch(input);

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
}

#[derive(Clone, Debug, PartialEq)]
pub struct BinaryExpr {
    pub(super) lhs: Arc<Expr>,
    pub(super) rhs: Arc<Expr>,
    pub(super) op: BinaryOp,
}

impl BinaryExpr {
    pub fn new(lhs: Expr, rhs: Expr, op: BinaryOp) -> Self {
        Self {
            lhs: Arc::new(lhs),
            rhs: Arc::new(rhs),
            op,
        }
    }
}

impl<T> ExprQuery<T> for BinaryExpr
where
    T: ExprProjection,
{
    fn dispatch<'a>(&'a mut self, input: &T) -> AnyValue<'a> {
        let lhs = Arc::make_mut(&mut self.lhs).dispatch(input);
        let rhs = Arc::make_mut(&mut self.rhs).dispatch(input);

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
    pub(super) first: Arc<Expr>,
    pub(super) second: Arc<Expr>,
    pub(super) third: Arc<Expr>,
    pub(super) operation: TrinaryOp,
}

impl TrinaryExpr {
    pub fn new(first: Expr, second: Expr, third: Expr, operation: TrinaryOp) -> Self {
        Self {
            first: Arc::new(first),
            second: Arc::new(second),
            third: Arc::new(third),
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
                let condition = Arc::make_mut(&mut self.first).dispatch(input);

                let cond = match condition {
                    AnyValue::Bool(b) => b,
                    _ => return AnyValue::Null,
                };

                if cond {
                    Arc::make_mut(&mut self.second).dispatch(input)
                } else {
                    Arc::make_mut(&mut self.third).dispatch(input)
                }
            }
            TrinaryOp::Clamp => {
                let value = Arc::make_mut(&mut self.first)
                    .dispatch(input)
                    .extract::<f32>();
                let min = Arc::make_mut(&mut self.second)
                    .dispatch(input)
                    .extract::<f32>();
                let max = Arc::make_mut(&mut self.third)
                    .dispatch(input)
                    .extract::<f32>();

                match (value, min, max) {
                    (Some(value), Some(min), Some(max)) => AnyValue::Float32(value.clamp(min, max)),
                    _ => AnyValue::Null,
                }
            }
        }
    }
}
