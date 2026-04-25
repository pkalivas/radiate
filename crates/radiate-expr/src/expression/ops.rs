use crate::{AnyValue, DataType, Expr, ExprProjection, ExprQuery, ExprResult};
use radiate_error::radiate_bail;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub enum UnaryOp {
    Not,
    Neg,
    Abs,
    Cast(DataType),
    Debug,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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
    fn dispatch<'a>(&'a mut self, input: &T) -> ExprResult<'a> {
        let value = self.child.dispatch(input)?;

        match self.op {
            UnaryOp::Not => match value {
                AnyValue::Bool(b) => Ok(AnyValue::Bool(!b)),
                _ => radiate_bail!(Expr: "Logical NOT is only supported for boolean types"),
            },
            UnaryOp::Neg => match value.extract::<f32>() {
                Some(v) => Ok(AnyValue::Float32(-v)),
                None => radiate_bail!(Expr: "Negation is only supported for numeric types"),
            },
            UnaryOp::Abs => match value.extract::<f32>() {
                Some(v) => Ok(AnyValue::Float32(v.abs())),
                None => radiate_bail!(Expr: "Absolute value is only supported for numeric types"),
            },
            UnaryOp::Cast(ref to) => match value.clone().cast(&to) {
                Some(v) => Ok(v),
                None => radiate_bail!(Expr: "Failed to cast value {:?} to type {:?}", value, to),
            },
            UnaryOp::Debug => {
                println!("{:?}", value);
                Ok(value)
            }
        }
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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
    Pow,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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
    fn dispatch<'a>(&'a mut self, input: &T) -> ExprResult<'a> {
        let lhs = self.lhs.dispatch(input)?;
        let rhs = self.rhs.dispatch(input)?;

        // println!("LHS: {:?}, RHS: {:?}, LHS < RHS: {:?}", lhs, rhs, lhs < rhs);

        let result = match self.op {
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
            BinaryOp::Pow => crate::datatype::pow_anyvalue(&lhs, &rhs)?,
        };

        Ok(result)
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TrinaryOp {
    If,
    Clamp,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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
    fn dispatch<'a>(&'a mut self, input: &T) -> ExprResult<'a> {
        match self.operation {
            TrinaryOp::If => {
                let condition = self.first.dispatch(input)?;

                let cond = match condition {
                    AnyValue::Bool(b) => b,
                    _ => radiate_bail!(Expr: "Condition must be a boolean"),
                };

                if cond {
                    self.second.dispatch(input)
                } else {
                    self.third.dispatch(input)
                }
            }
            TrinaryOp::Clamp => {
                let value = self.first.dispatch(input)?.extract::<f32>();
                let min = self.second.dispatch(input)?.extract::<f32>();
                let max = self.third.dispatch(input)?.extract::<f32>();

                if value.is_none() {
                    return Ok(AnyValue::Null);
                }

                match (value, min, max) {
                    (Some(value), Some(min), Some(max)) => {
                        Ok(AnyValue::Float32(value.clamp(min, max)))
                    }
                    _ => radiate_bail!(Expr: "Clamp operation requires numeric values"),
                }
            }
        }
    }
}
