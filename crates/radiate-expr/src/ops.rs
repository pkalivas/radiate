use radiate_utils::{DataType, SmallStr};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, PartialEq)]
pub enum ScheduleOp {
    Interval {
        count: usize,
        limit: usize,
    },
    Duration {
        #[cfg_attr(feature = "serde", serde(skip))]
        last: Option<std::time::Instant>,
        interval: std::time::Duration,
    },
}

impl Debug for ScheduleOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScheduleOp::Interval { count, limit } => {
                write!(f, "Interval {{ count: {}, limit: {} }}", count, limit)
            }
            ScheduleOp::Duration { last, interval } => {
                write!(
                    f,
                    "Duration {{ last: {:?}, interval: {:?} }}",
                    last, interval
                )
            }
        }
    }
}

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
pub enum RollupOp {
    First,
    Last,
    Mean,
    StdDev,
    Min,
    Max,
    Sum,
    Var,
    Skew,
    Count,
    Unique,
    Slope,
    Quantile(f32),
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
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TrinaryOp {
    If,
    Clamp,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, PartialEq)]
pub enum SelectOp {
    Identity,
    Index(usize),
    Range(usize, usize),
    Field(SmallStr),
    Nested {
        parent: Box<SelectOp>,
        child: Box<SelectOp>,
    },
}

impl From<usize> for SelectOp {
    fn from(idx: usize) -> Self {
        SelectOp::Index(idx)
    }
}

impl From<std::ops::Range<usize>> for SelectOp {
    fn from(range: std::ops::Range<usize>) -> Self {
        SelectOp::Range(range.start, range.end)
    }
}

impl From<SmallStr> for SelectOp {
    fn from(field: SmallStr) -> Self {
        SelectOp::Field(field)
    }
}

impl From<(SmallStr, SmallStr)> for SelectOp {
    fn from((parent, child): (SmallStr, SmallStr)) -> Self {
        SelectOp::Nested {
            parent: Box::new(SelectOp::Field(parent)),
            child: Box::new(SelectOp::Field(child)),
        }
    }
}

impl Debug for SelectOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SelectOp::Identity => write!(f, "Identity"),
            SelectOp::Index(idx) => write!(f, "Index({})", idx),
            SelectOp::Range(start, end) => write!(f, "Range({},{})", start, end),
            SelectOp::Field(field) => write!(f, "Field({})", field),
            SelectOp::Nested { parent, child } => {
                write!(f, "Nested({:?}, {:?})", parent, child)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Expr, ExprResult, ProjectExpr, ops::SelectOp};
    use radiate_error::RadiateError;
    use radiate_utils::AnyValue;

    struct NullMetrics;
    impl<'a> ProjectExpr<'a> for NullMetrics {
        fn select(&'a self, _: &SelectOp) -> Result<AnyValue<'a>, RadiateError> {
            Ok(AnyValue::Null)
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
