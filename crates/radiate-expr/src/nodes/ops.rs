use crate::ExprSelector;
use crate::{Evaluate, Expr, ExprResult};
use radiate_error::radiate_bail;
use radiate_utils::{AnyValue, DataType};
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
    /// Fused affine: `scale * child + bias`. Replaces the `.mul(lit).add(lit)`
    /// pattern with a single node. Chains collapse via [`fuse_affine`].
    Affine {
        scale: f32,
        bias: f32,
    },
    Stagnation {
        epsilon: f32,
        last_value: Option<f32>,
        count: u32,
    },
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct UnaryExpr {
    pub(crate) child: Box<Expr>,
    pub(crate) op: UnaryOp,
}

impl UnaryExpr {
    pub fn new(child: Expr, op: UnaryOp) -> Self {
        Self {
            child: Box::new(child),
            op,
        }
    }

    pub fn reset(&mut self) {
        self.child.reset();
        match &mut self.op {
            UnaryOp::Stagnation {
                last_value, count, ..
            } => {
                *last_value = None;
                *count = 0;
            }
            _ => {}
        }
    }
}

impl<'a, T> Evaluate<'a, T> for UnaryExpr
where
    T: ExprSelector,
{
    fn eval(&'a mut self, metrics: &T) -> ExprResult<'a> {
        let value = self.child.eval(metrics)?;

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
            UnaryOp::Cast(ref to) => match value.clone().cast(to) {
                Some(v) => Ok(v),
                None => radiate_bail!(Expr: "Failed to cast value {:?} to type {:?}", value, to),
            },
            UnaryOp::Debug => {
                println!("{:?}", value);
                Ok(value)
            }
            UnaryOp::Affine { scale, bias } => match value.extract::<f32>() {
                Some(x) if x.is_finite() => Ok(AnyValue::Float32(scale * x + bias)),
                _ => Ok(AnyValue::Null),
            },
            UnaryOp::Stagnation {
                epsilon,
                ref mut last_value,
                ref mut count,
            } => {
                let current = match value.extract::<f32>() {
                    Some(v) if v.is_finite() => v,
                    _ => return Ok(AnyValue::Null),
                };

                match last_value {
                    None => {
                        *last_value = Some(current);
                        *count = 0;
                    }
                    Some(last) => {
                        if (current - *last).abs() > epsilon {
                            *last_value = Some(current);
                            *count = 0;
                        } else {
                            *count = count.saturating_add(1);
                        }
                    }
                }

                Ok(AnyValue::UInt32(*count))
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
    /// Returns lhs if finite, otherwise rhs. Treats Null, NaN, ±Inf as fallback triggers.
    Coalesce,
    /// Elementwise min of two numeric values. NaN-on-one-side returns the other.
    Min,
    /// Elementwise max of two numeric values. NaN-on-one-side returns the other.
    Max,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct BinaryExpr {
    pub(crate) lhs: Box<Expr>,
    pub(crate) rhs: Box<Expr>,
    pub(crate) op: BinaryOp,
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

impl<'a, T> Evaluate<'a, T> for BinaryExpr
where
    T: ExprSelector,
{
    fn eval(&'a mut self, metrics: &T) -> ExprResult<'a> {
        // Coalesce short-circuits: only evaluate rhs when lhs is bad.
        if let BinaryOp::Coalesce = self.op {
            let lhs = self.lhs.eval(metrics)?;
            let is_bad = match lhs.extract::<f32>() {
                Some(v) => !v.is_finite(),
                None => matches!(lhs, AnyValue::Null),
            };
            return if is_bad {
                self.rhs.eval(metrics)
            } else {
                Ok(lhs)
            };
        }

        let lhs = self.lhs.eval(metrics)?;
        let rhs = self.rhs.eval(metrics)?;

        let result = match self.op {
            BinaryOp::Coalesce => unreachable!("handled above"),
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
            BinaryOp::Pow => radiate_utils::pow_anyvalue(&lhs, &rhs)?,
            BinaryOp::Min => match (lhs.extract::<f32>(), rhs.extract::<f32>()) {
                (Some(a), Some(b)) => AnyValue::Float32(a.min(b)),
                _ => radiate_bail!(Expr: "Min requires numeric operands"),
            },
            BinaryOp::Max => match (lhs.extract::<f32>(), rhs.extract::<f32>()) {
                (Some(a), Some(b)) => AnyValue::Float32(a.max(b)),
                _ => radiate_bail!(Expr: "Max requires numeric operands"),
            },
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
    pub(crate) first: Box<Expr>,
    pub(crate) second: Box<Expr>,
    pub(crate) third: Box<Expr>,
    pub(crate) operation: TrinaryOp,
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

impl<'a, T> Evaluate<'a, T> for TrinaryExpr
where
    T: ExprSelector,
{
    fn eval(&'a mut self, metrics: &T) -> ExprResult<'a> {
        match self.operation {
            TrinaryOp::If => {
                let condition = self.first.eval(metrics)?;

                let cond = match condition {
                    AnyValue::Bool(b) => b,
                    _ => radiate_bail!(Expr: "Condition must be a boolean"),
                };

                if cond {
                    self.second.eval(metrics)
                } else {
                    self.third.eval(metrics)
                }
            }
            TrinaryOp::Clamp => {
                let value = self.first.eval(metrics)?.extract::<f32>();
                let min = self.second.eval(metrics)?.extract::<f32>();
                let max = self.third.eval(metrics)?.extract::<f32>();

                let (min_v, max_v) = match (min, max) {
                    (Some(a), Some(b)) => (a, b),
                    _ => radiate_bail!(Expr: "Clamp bounds must be numeric"),
                };

                // Null, NaN, ±Inf all fall back to the floor — the safer default
                // for rate-style controllers where a runaway high value is worse
                // than a conservative low one.
                let result = match value {
                    Some(v) if v.is_finite() => v.clamp(min_v, max_v),
                    _ => min_v,
                };
                Ok(AnyValue::Float32(result))
            }
        }
    }
}

/// Construct `Unary(Affine(scale * child + bias))`, collapsing nested affines.
/// `scale * (s2 * x + b2) + bias = (scale * s2) * x + (scale * b2 + bias)`.
///
/// Shared between the `.affine(...)` builder and the compile-pass binary-fusion
/// rewriters so both produce the same fused shape.
pub(crate) fn fuse_affine(child: Expr, scale: f32, bias: f32) -> Expr {
    use crate::expr::ExprKind;
    if let ExprKind::Unary(u) = child.kind {
        if matches!(u.op, UnaryOp::Affine { .. }) {
            let UnaryExpr { child: inner, op } = u;
            let UnaryOp::Affine {
                scale: s2,
                bias: b2,
            } = op
            else {
                unreachable!()
            };

            return Expr::new(ExprKind::Unary(UnaryExpr::new(
                *inner,
                UnaryOp::Affine {
                    scale: scale * s2,
                    bias: scale * b2 + bias,
                },
            )));
        }

        return Expr::new(ExprKind::Unary(UnaryExpr::new(
            Expr::new(ExprKind::Unary(u)),
            UnaryOp::Affine { scale, bias },
        )));
    }

    Expr::new(ExprKind::Unary(UnaryExpr::new(
        child,
        UnaryOp::Affine { scale, bias },
    )))
}
