use crate::ExprSelect;
use crate::{EvalExpr, Expr, ExprResult};
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
        if let UnaryOp::Stagnation {
            last_value, count, ..
        } = &mut self.op
        {
            *last_value = None;
            *count = 0;
        }
    }
}

impl<'a, T> EvalExpr<'a, T> for UnaryExpr
where
    T: ExprSelect<'a>,
{
    fn evaluate(&'a mut self, metrics: &'a T) -> ExprResult<'a> {
        let value = self.child.evaluate(metrics)?;

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

impl<'a, T> EvalExpr<'a, T> for BinaryExpr
where
    T: ExprSelect<'a>,
{
    fn evaluate(&'a mut self, metrics: &'a T) -> ExprResult<'a> {
        // Coalesce short-circuits: only evaluate rhs when lhs is bad.
        if let BinaryOp::Coalesce = self.op {
            let lhs = self.lhs.evaluate(metrics)?;
            let is_bad = match lhs.extract::<f32>() {
                Some(v) => !v.is_finite(),
                None => matches!(lhs, AnyValue::Null),
            };
            return if is_bad {
                self.rhs.evaluate(metrics)
            } else {
                Ok(lhs)
            };
        }

        let lhs = self.lhs.evaluate(metrics)?;
        let rhs = self.rhs.evaluate(metrics)?;

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

impl<'a, T> EvalExpr<'a, T> for TrinaryExpr
where
    T: ExprSelect<'a>,
{
    fn evaluate(&'a mut self, metrics: &'a T) -> ExprResult<'a> {
        match self.operation {
            TrinaryOp::If => {
                let condition = self.first.evaluate(metrics)?;

                let cond = match condition {
                    AnyValue::Bool(b) => b,
                    _ => radiate_bail!(Expr: "Condition must be a boolean"),
                };

                if cond {
                    self.second.evaluate(metrics)
                } else {
                    self.third.evaluate(metrics)
                }
            }
            TrinaryOp::Clamp => {
                let value = self.first.evaluate(metrics)?.extract::<f32>();
                let min = self.second.evaluate(metrics)?.extract::<f32>();
                let max = self.third.evaluate(metrics)?.extract::<f32>();

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
    use crate::query::ExprNode;
    if let ExprNode::Unary(u) = child.node {
        if matches!(u.op, UnaryOp::Affine { .. }) {
            let UnaryExpr { child: inner, op } = u;
            let UnaryOp::Affine {
                scale: s2,
                bias: b2,
            } = op
            else {
                unreachable!()
            };

            return Expr::new(ExprNode::Unary(UnaryExpr::new(
                *inner,
                UnaryOp::Affine {
                    scale: scale * s2,
                    bias: scale * b2 + bias,
                },
            )));
        }

        return Expr::new(ExprNode::Unary(UnaryExpr::new(
            Expr::new(ExprNode::Unary(u)),
            UnaryOp::Affine { scale, bias },
        )));
    }

    Expr::new(ExprNode::Unary(UnaryExpr::new(
        child,
        UnaryOp::Affine { scale, bias },
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Selector;

    struct NullMetrics;
    impl<'a> ExprSelect<'a> for NullMetrics {
        fn select(&'a self, _sel: &Selector) -> AnyValue<'a> {
            AnyValue::Null
        }
    }

    fn eval(mut expr: Expr) -> ExprResult<'static> {
        let metrics = NullMetrics;
        expr.evaluate(&metrics).map(|v| v.into_static())
    }

    #[test]
    fn not_negates_bool() {
        assert_eq!(eval(Expr::lit(true).not()).unwrap(), AnyValue::Bool(false));
    }

    #[test]
    fn not_on_non_bool_errors() {
        assert!(eval(Expr::lit(5.0_f32).not()).is_err());
    }

    #[test]
    fn neg_and_abs_on_numeric() {
        assert_eq!(
            eval(Expr::lit(3.0_f32).neg()).unwrap(),
            AnyValue::Float32(-3.0)
        );
        assert_eq!(
            eval(Expr::lit(-3.0_f32).abs()).unwrap(),
            AnyValue::Float32(3.0)
        );
    }

    #[test]
    fn cast_changes_dtype() {
        assert_eq!(
            eval(Expr::lit(3.0_f32).cast(DataType::UInt64)).unwrap(),
            AnyValue::UInt64(3)
        );
    }

    #[test]
    fn stagnation_counts_consecutive_small_changes() {
        let mut expr = Expr::lit(1.0_f32).stagnation(0.1);
        let metrics = NullMetrics;
        assert_eq!(
            expr.evaluate(&metrics).unwrap().into_static(),
            AnyValue::UInt32(0)
        );
        assert_eq!(
            expr.evaluate(&metrics).unwrap().into_static(),
            AnyValue::UInt32(1)
        );
        assert_eq!(
            expr.evaluate(&metrics).unwrap().into_static(),
            AnyValue::UInt32(2)
        );
    }

    #[test]
    fn binary_arithmetic_and_comparison() {
        assert_eq!(
            eval(Expr::lit(2_i32).add(3_i32)).unwrap(),
            AnyValue::Int32(5)
        );
        assert_eq!(
            eval(Expr::lit(2_i32).lt(3_i32)).unwrap(),
            AnyValue::Bool(true)
        );
        assert_eq!(
            eval(Expr::lit(2_i32).eq(2_i32)).unwrap(),
            AnyValue::Bool(true)
        );
    }

    #[test]
    fn coalesce_short_circuits_on_finite_lhs() {
        // rhs would error if evaluated (Not on a float) — if coalesce ever
        // stops short-circuiting, this test starts failing with an Err.
        let result = eval(Expr::lit(2.0_f32).coalesce(Expr::lit(9.0_f32).not())).unwrap();
        assert_eq!(result, AnyValue::Float32(2.0));
    }

    #[test]
    fn coalesce_falls_back_to_rhs_on_nan() {
        let result = eval(Expr::lit(f32::NAN).coalesce(Expr::lit(7.0_f32))).unwrap();
        assert_eq!(result, AnyValue::Float32(7.0));
    }

    #[test]
    fn min_with_and_max_with() {
        assert_eq!(
            eval(Expr::lit(2.0_f32).min_with(5.0_f32)).unwrap(),
            AnyValue::Float32(2.0)
        );
        assert_eq!(
            eval(Expr::lit(2.0_f32).max_with(5.0_f32)).unwrap(),
            AnyValue::Float32(5.0)
        );
    }

    #[test]
    fn trinary_if_picks_correct_branch() {
        let then_expr = Expr::when(Expr::lit(true)).then(1_i32).otherwise(2_i32);
        assert_eq!(eval(then_expr).unwrap(), AnyValue::Int32(1));

        let else_expr = Expr::when(Expr::lit(false)).then(1_i32).otherwise(2_i32);
        assert_eq!(eval(else_expr).unwrap(), AnyValue::Int32(2));
    }

    #[test]
    fn trinary_if_requires_bool_condition() {
        let expr = Expr::when(Expr::lit(1_i32)).then(1_i32).otherwise(2_i32);
        assert!(eval(expr).is_err());
    }

    #[test]
    fn clamp_bounds_finite_value() {
        let expr = Expr::lit(15.0_f32).clamp(0.0_f32, 10.0_f32);
        assert_eq!(eval(expr).unwrap(), AnyValue::Float32(10.0));
    }

    #[test]
    fn clamp_falls_back_to_floor_on_nan() {
        let expr = Expr::lit(f32::NAN).clamp(1.0_f32, 10.0_f32);
        assert_eq!(eval(expr).unwrap(), AnyValue::Float32(1.0));
    }
}
