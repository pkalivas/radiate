use super::{Evaluate, Expr, ExprResult};
use crate::MetricSet;
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

impl Evaluate for UnaryExpr {
    fn eval<'a>(&'a mut self, metrics: &MetricSet) -> ExprResult<'a> {
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

impl Evaluate for BinaryExpr {
    fn eval<'a>(&'a mut self, metrics: &MetricSet) -> ExprResult<'a> {
        // Coalesce short-circuits: only evaluate rhs when lhs is bad.
        if let BinaryOp::Coalesce = self.op {
            let lhs = self.lhs.eval(metrics)?;
            let is_bad = match lhs.extract::<f32>() {
                Some(v) => !v.is_finite(),
                None => matches!(lhs, AnyValue::Null),
            };
            return if is_bad { self.rhs.eval(metrics) } else { Ok(lhs) };
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

impl Evaluate for TrinaryExpr {
    fn eval<'a>(&'a mut self, metrics: &MetricSet) -> ExprResult<'a> {
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

/// `scale * child + bias`. A fused affine operator — replaces the
/// `.mul(lit).add(lit)` pattern with a single node carrying two constants.
///
/// Useful for normalization (`(x - μ) / σ`), gain-style controllers, and any
/// linear remap of a metric. Chains fuse algebraically when constructed via
/// the `.affine(...)` builder, so `x.affine(a, b).affine(c, d)` collapses to a
/// single `Affine { scale: c*a, bias: c*b + d }`.
///
/// Non-finite child output (Null, NaN, ±Inf) is propagated as `Null` so the
/// outer Clamp or Coalesce can take over.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct AffineExpr {
    pub(super) child: Box<Expr>,
    pub(super) scale: f32,
    pub(super) bias: f32,
}

impl AffineExpr {
    pub fn new(child: Expr, scale: f32, bias: f32) -> Self {
        Self {
            child: Box::new(child),
            scale,
            bias,
        }
    }

    pub(super) fn child_mut(&mut self) -> &mut Expr {
        &mut self.child
    }

    pub(super) fn into_parts(self) -> (Box<Expr>, f32, f32) {
        (self.child, self.scale, self.bias)
    }
}

impl Evaluate for AffineExpr {
    fn eval<'a>(&'a mut self, metrics: &MetricSet) -> ExprResult<'a> {
        let value = self.child.eval(metrics)?;
        match value.extract::<f32>() {
            Some(x) if x.is_finite() => Ok(AnyValue::Float32(self.scale * x + self.bias)),
            _ => Ok(AnyValue::Null),
        }
    }
}
