mod agg;
mod compare;
mod condition;
mod logic;
mod metric;
mod scalar;

use crate::{MetricSet, Valid};
use std::fmt::Debug;

pub use agg::{AggregateExprNode, Rollup};
pub use compare::{CompareExprNode, CompareOp};
pub use condition::{Then, When};
pub use logic::{LogicExprNode, LogicOp};
pub use metric::MetricExprNode;
pub use scalar::Value;

pub trait ExprEval<I> {
    fn eval<'a>(&'a mut self, input: &'a I) -> Value<'a>;
}

#[derive(Clone)]
pub enum ExprNode<'a> {
    Value(Value<'a>),
    Select(MetricExprNode),
    Aggregate(AggregateExprNode<'a>),
    Compare(CompareExprNode<'a>),
    Logic(LogicExprNode<'a>),

    If {
        cond: Box<ExprNode<'a>>,
        then_expr: Box<ExprNode<'a>>,
        else_expr: Box<ExprNode<'a>>,
    },
}

impl<'a> ExprNode<'a> {
    pub fn rolling(self, window_size: usize) -> Self {
        ExprNode::Aggregate(AggregateExprNode::new(self, Rollup::Mean, window_size))
    }

    pub fn n_unique(self, len: usize) -> Self {
        ExprNode::Aggregate(AggregateExprNode::new(self, Rollup::NUnique, len))
    }

    pub fn mean(self) -> Self {
        match self {
            ExprNode::Select(node) => ExprNode::Select(node.mean()),
            ExprNode::Aggregate(node) => node.mean(),
            other => other,
        }
    }

    pub fn stddev(self) -> Self {
        match self {
            ExprNode::Select(node) => ExprNode::Select(node.stddev()),
            ExprNode::Aggregate(node) => node.stddev(),
            other => other,
        }
    }

    pub fn min(self) -> Self {
        match self {
            ExprNode::Select(node) => ExprNode::Select(node.min()),
            ExprNode::Aggregate(node) => node.min(),
            other => other,
        }
    }

    pub fn max(self) -> Self {
        match self {
            ExprNode::Select(node) => ExprNode::Select(node.max()),
            ExprNode::Aggregate(node) => node.max(),
            other => other,
        }
    }

    pub fn sum(self) -> Self {
        match self {
            ExprNode::Select(node) => ExprNode::Select(node.sum()),
            ExprNode::Aggregate(node) => node.sum(),
            other => other,
        }
    }

    pub fn count(self) -> Self {
        match self {
            ExprNode::Select(node) => ExprNode::Select(node.count()),
            ExprNode::Aggregate(node) => node.count(),
            other => other,
        }
    }

    pub fn lt(self, rhs: impl Into<ExprNode<'a>>) -> Self {
        ExprNode::Compare(CompareExprNode::new(self, rhs.into(), CompareOp::Lt))
    }

    pub fn lte(self, rhs: impl Into<ExprNode<'a>>) -> Self {
        ExprNode::Compare(CompareExprNode::new(self, rhs.into(), CompareOp::Lte))
    }

    pub fn gt(self, rhs: impl Into<ExprNode<'a>>) -> Self {
        ExprNode::Compare(CompareExprNode::new(self, rhs.into(), CompareOp::Gt))
    }

    pub fn gte(self, rhs: impl Into<ExprNode<'a>>) -> Self {
        ExprNode::Compare(CompareExprNode::new(self, rhs.into(), CompareOp::Gte))
    }

    pub fn eq(self, rhs: impl Into<ExprNode<'a>>) -> Self {
        ExprNode::Compare(CompareExprNode::new(self, rhs.into(), CompareOp::Eq))
    }

    pub fn and(self, rhs: impl Into<ExprNode<'a>>) -> Self {
        ExprNode::Logic(LogicExprNode::new(self, rhs.into(), LogicOp::And))
    }

    pub fn or(self, rhs: impl Into<ExprNode<'a>>) -> Self {
        ExprNode::Logic(LogicExprNode::new(self, rhs.into(), LogicOp::Or))
    }
}

impl From<MetricExprNode> for ExprNode<'_> {
    fn from(node: MetricExprNode) -> Self {
        ExprNode::Select(node)
    }
}

impl<'a> From<Value<'a>> for ExprNode<'a> {
    fn from(value: Value<'a>) -> Self {
        ExprNode::Value(value)
    }
}

impl<'a> From<f32> for ExprNode<'a> {
    fn from(value: f32) -> Self {
        ExprNode::Value(Value::Float32(value))
    }
}

impl<'a> From<i32> for ExprNode<'a> {
    fn from(value: i32) -> Self {
        ExprNode::Value(Value::Int32(value))
    }
}

impl<'a> From<bool> for ExprNode<'a> {
    fn from(value: bool) -> Self {
        ExprNode::Value(Value::Bool(value))
    }
}

impl ExprEval<MetricSet> for ExprNode<'_> {
    fn eval<'a>(&'a mut self, metrics: &'a MetricSet) -> Value<'a> {
        match self {
            ExprNode::Value(s) => s.clone(),
            ExprNode::Select(m) => m.eval(metrics),
            ExprNode::Aggregate(a) => a.eval(metrics),
            ExprNode::Compare(c) => c.eval(metrics),
            ExprNode::Logic(l) => l.eval(metrics),

            ExprNode::If {
                cond,
                then_expr,
                else_expr,
            } => {
                if cond.eval(metrics).as_bool() {
                    then_expr.eval(metrics)
                } else {
                    else_expr.eval(metrics)
                }
            }
        }
    }
}

impl Valid for ExprNode<'_> {
    fn is_valid(&self) -> bool {
        true
    }
}

impl<'a> Debug for ExprNode<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExprNode::Value(v) => write!(f, "{v:?}"),
            ExprNode::Select(_) => write!(f, "Metric)"),
            ExprNode::Aggregate(_) => write!(f, "Aggregate"),
            ExprNode::Compare(_) => write!(f, "Compare"),
            ExprNode::Logic(_) => write!(f, "Logic"),

            ExprNode::If {
                cond,
                then_expr,
                else_expr,
            } => write!(f, "if {cond:?} then {then_expr:?} else {else_expr:?}"),
        }
    }
}

impl<'a> PartialEq for ExprNode<'a> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ExprNode::Value(a), ExprNode::Value(b)) => a == b,
            _ => false,
        }
    }
}

pub mod expr {
    use crate::{ExprNode, MetricExprNode, Value, When};

    pub fn val<'a>(scalar: impl Into<Value<'a>>) -> ExprNode<'a> {
        ExprNode::Value(scalar.into())
    }

    pub fn metric(name: &'static str) -> MetricExprNode {
        MetricExprNode::new(name)
    }

    pub fn rolling<'a>(child: impl Into<ExprNode<'a>>, window_size: usize) -> ExprNode<'a> {
        child.into().rolling(window_size)
    }

    pub fn stagnant<'a>(child: impl Into<ExprNode<'a>>, len: usize) -> ExprNode<'a> {
        child.into().n_unique(len).lte(1.0)
    }

    pub fn when<'a>(cond: impl Into<ExprNode<'a>>) -> When<'a> {
        When::new(cond)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ExprEval, Rate, Value};

    fn metrics_with_values(name: &'static str, values: &[f32]) -> MetricSet {
        let mut metrics = MetricSet::new();
        for &value in values {
            metrics.upsert((name, value));
        }

        metrics
    }

    fn metrics_with_two(
        a_name: &'static str,
        a_values: &[f32],
        b_name: &'static str,
        b_values: &[f32],
    ) -> MetricSet {
        let mut metrics = MetricSet::new();
        for &value in a_values {
            metrics.upsert((a_name, value));
        }
        for &value in b_values {
            metrics.upsert((b_name, value));
        }

        metrics
    }

    #[test]
    fn and_works() {
        let mut metrics = MetricSet::new();
        metrics.upsert(("NAME", 1.0));
        metrics.upsert(("NAME", 2.0));
        metrics.upsert(("NAME", 3.0));
        metrics.upsert(("NAME", 4.0));

        metrics.upsert(("OTHER", 0.0));
        metrics.upsert(("OTHER", 0.5));

        let mut expr = expr::metric("NAME")
            .mean()
            .lt(3.0)
            .and(expr::metric("OTHER").sum().lt(0.6));

        assert_eq!(expr.eval(&metrics), Value::Bool(true));
    }

    #[test]
    fn and_false_when_rhs_false() {
        let mut metrics = MetricSet::new();
        metrics.upsert(("NAME", 1.0));
        metrics.upsert(("NAME", 2.0));
        metrics.upsert(("NAME", 3.0));
        metrics.upsert(("NAME", 4.0));

        metrics.upsert(("OTHER", 0.2));
        metrics.upsert(("OTHER", 0.2));

        let mut expr = expr::metric("NAME")
            .mean()
            .lt(3.0)
            .and(expr::metric("OTHER").stddev().gt(0.1));

        assert_eq!(expr.eval(&metrics), Value::Bool(false));
    }

    #[test]
    fn reads_like_polars() {
        let mut metrics = MetricSet::new();
        metrics.upsert(("NAME", 1.0));
        metrics.upsert(("NAME", 2.0));
        metrics.upsert(("NAME", 3.0));
        metrics.upsert(("NAME", 4.0));

        metrics.upsert(("OTHER", 0.0));
        metrics.upsert(("OTHER", 0.5));

        let mut expr = expr::metric("NAME")
            .mean()
            .rolling(3)
            .lt(3.0)
            .and(expr::metric("OTHER").stddev().gt(0.1));

        let _ = expr.eval(&metrics);
    }

    #[test]
    fn metric_mean_lt_scalar_true() {
        let metrics = metrics_with_values("NAME", &[1.0, 2.0, 3.0, 4.0]);

        let mut expr = expr::metric("NAME").mean().lt(3.0);

        assert_eq!(expr.eval(&metrics), Value::Bool(true));
    }

    #[test]
    fn metric_mean_lt_scalar_false() {
        let metrics = metrics_with_values("NAME", &[2.0, 3.0, 4.0, 5.0]);

        let mut expr = expr::metric("NAME").mean().lt(3.0);

        assert_eq!(expr.eval(&metrics), Value::Bool(false));
    }

    #[test]
    fn metric_mean_gt_scalar_true() {
        let metrics = metrics_with_values("NAME", &[2.0, 3.0, 4.0, 5.0]);

        let mut expr = expr::metric("NAME").mean().gt(3.0);

        assert_eq!(expr.eval(&metrics), Value::Bool(true));
    }

    #[test]
    fn metric_mean_gt_scalar_false() {
        let metrics = metrics_with_values("NAME", &[1.0, 2.0, 3.0, 4.0]);

        let mut expr = expr::metric("NAME").mean().gt(3.0);

        assert_eq!(expr.eval(&metrics), Value::Bool(false));
    }

    #[test]
    fn metric_eq_scalar_true() {
        let metrics = metrics_with_values("NAME", &[1.0, 2.0, 3.0, 4.0]);

        let mut expr = expr::metric("NAME").mean().eq(2.5);

        assert_eq!(expr.eval(&metrics), Value::Bool(true));
    }

    #[test]
    fn and_true_when_both_sides_true() {
        let metrics = metrics_with_two("NAME", &[1.0, 2.0, 3.0, 4.0], "OTHER", &[0.0, 0.5]);

        let mut expr = expr::metric("NAME")
            .mean()
            .lt(3.0)
            .and(expr::metric("OTHER").stddev().gt(0.1));

        assert_eq!(expr.eval(&metrics), Value::Bool(true));
    }

    #[test]
    fn and_false_when_left_false() {
        let metrics = metrics_with_two("NAME", &[5.0, 6.0, 7.0, 8.0], "OTHER", &[0.0, 0.5]);

        let mut expr = expr::metric("NAME")
            .mean()
            .lt(3.0)
            .and(expr::metric("OTHER").stddev().gt(0.1));

        assert_eq!(expr.eval(&metrics), Value::Bool(false));
    }

    #[test]
    fn and_false_when_right_false() {
        let metrics = metrics_with_two("NAME", &[1.0, 2.0, 3.0, 4.0], "OTHER", &[0.2, 0.2]);

        let mut expr = expr::metric("NAME")
            .mean()
            .lt(3.0)
            .and(expr::metric("OTHER").stddev().gt(0.1));

        assert_eq!(expr.eval(&metrics), Value::Bool(false));
    }

    #[test]
    fn or_true_when_left_true() {
        let metrics = metrics_with_two("NAME", &[1.0, 2.0, 3.0, 4.0], "OTHER", &[0.2, 0.2]);

        let mut expr = expr::metric("NAME")
            .mean()
            .lt(3.0)
            .or(expr::metric("OTHER").stddev().gt(0.1));

        assert_eq!(expr.eval(&metrics), Value::Bool(true));
    }

    #[test]
    fn or_true_when_right_true() {
        let metrics = metrics_with_two("NAME", &[5.0, 6.0, 7.0, 8.0], "OTHER", &[0.0, 0.5]);

        let mut expr = expr::metric("NAME")
            .mean()
            .lt(3.0)
            .or(expr::metric("OTHER").stddev().gt(0.1));

        assert_eq!(expr.eval(&metrics), Value::Bool(true));
    }

    #[test]
    fn or_false_when_both_false() {
        let metrics = metrics_with_two("NAME", &[5.0, 6.0, 7.0, 8.0], "OTHER", &[0.2, 0.2]);

        let mut expr = expr::metric("NAME")
            .mean()
            .lt(3.0)
            .or(expr::metric("OTHER").stddev().gt(0.1));

        assert_eq!(expr.eval(&metrics), Value::Bool(false));
    }

    #[test]
    fn when_then_otherwise_takes_then_branch() {
        let metrics = metrics_with_values("NAME", &[1.0, 2.0, 3.0, 4.0]);

        let mut expr = expr::when(expr::metric("NAME").mean().lt(3.0))
            .then(0.25)
            .otherwise(0.10);

        assert_eq!(expr.eval(&metrics), Value::Float32(0.25));
    }

    #[test]
    fn when_then_otherwise_takes_else_branch() {
        let metrics = metrics_with_values("NAME", &[5.0, 6.0, 7.0, 8.0]);

        let mut expr = expr::when(expr::metric("NAME").mean().lt(3.0))
            .then(0.25)
            .otherwise(0.10);

        assert_eq!(expr.eval(&metrics), Value::Float32(0.10));
    }

    #[test]
    fn when_then_otherwise_with_compound_condition() {
        let metrics = metrics_with_two("NAME", &[1.0, 2.0, 3.0, 4.0], "OTHER", &[0.0, 0.5]);

        let mut expr = expr::when(
            expr::metric("NAME")
                .mean()
                .lt(3.0)
                .and(expr::metric("OTHER").stddev().gt(0.1)),
        )
        .then(0.8)
        .otherwise(0.2);

        assert_eq!(expr.eval(&metrics), Value::Float32(0.8));
    }

    #[test]
    fn rolling_mean_lt_updates_over_multiple_evals() {
        let m1 = metrics_with_values("NAME", &[0.9]);
        let m2 = metrics_with_values("NAME", &[0.6]);
        let m3 = metrics_with_values("NAME", &[0.3]);
        let m4 = metrics_with_values("NAME", &[0.0]);

        let mut expr = expr::metric("NAME").rolling(3).mean().lt(0.5);

        assert_eq!(expr.eval(&m1), Value::Bool(false)); // 0.9
        assert_eq!(expr.eval(&m2), Value::Bool(false)); // (0.9 + 0.6)/2 = 0.75
        assert_eq!(expr.eval(&m3), Value::Bool(false)); // (0.9 + 0.6 + 0.3)/3 = 0.6
        assert_eq!(expr.eval(&m4), Value::Bool(true)); // (0.6 + 0.3 + 0.0)/3 = 0.3
    }

    #[test]
    fn rolling_compound_condition_reads_nicely() {
        let m1 = metrics_with_two("NAME", &[0.9], "OTHER", &[0.0, 0.5]);
        let m2 = metrics_with_two("NAME", &[0.6], "OTHER", &[0.0, 0.5]);
        let m3 = metrics_with_two("NAME", &[0.3], "OTHER", &[0.0, 0.5]);
        let m4 = metrics_with_two("NAME", &[0.0], "OTHER", &[0.0, 0.5]);

        let mut expr = expr::metric("NAME")
            .mean()
            .rolling(3)
            .lt(0.5)
            .and(expr::metric("OTHER").stddev().gt(0.1));

        assert_eq!(expr.eval(&m1), Value::Bool(false));
        assert_eq!(expr.eval(&m2), Value::Bool(false));
        assert_eq!(expr.eval(&m3), Value::Bool(false));
        assert_eq!(expr.eval(&m4), Value::Bool(true));
    }

    #[test]
    fn stagnant_expr_clamps_to_one_unique_value() {
        let mut metrics = metrics_with_values("NAME", &[1.0, 3.0, 3.3333, 33.0]);
        let mut expr = expr::metric("NAME").stagnant(3);

        metrics.upsert(("NAME", 1.0));
        assert_eq!(expr.eval(&metrics), Value::Bool(true));

        metrics.upsert(("NAME", 3.0));
        assert_eq!(expr.eval(&metrics), Value::Bool(false));

        metrics.upsert(("NAME", 3.3333));
        assert_eq!(expr.eval(&metrics), Value::Bool(false));

        metrics.upsert(("NAME", 33.0));
        assert_eq!(expr.eval(&metrics), Value::Bool(false));

        metrics.upsert(("NAME", 1.0));
        assert_eq!(expr.eval(&metrics), Value::Bool(false));

        metrics.upsert(("NAME", 1.0));
        assert_eq!(expr.eval(&metrics), Value::Bool(false));

        metrics.upsert(("NAME", 1.0));
        assert_eq!(expr.eval(&metrics), Value::Bool(true));
    }

    #[test]
    fn rate_expr_clamps_bool_to_numeric() {
        let metrics = metrics_with_values("NAME", &[1.0, 2.0, 3.0, 4.0]);

        let rate = Rate::Expr(
            expr::when(expr::metric("NAME").mean().lt(3.0))
                .then(0.8)
                .otherwise(0.1),
        );

        assert_eq!(rate.value_from_metrics(&metrics), 0.8);
    }
}
