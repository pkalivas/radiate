mod aggregate;
mod logical;
mod ops;
mod select;

use crate::AnyValue;

use aggregate::{AggExpr, BufferExpr, Rollup};
use logical::When;
use ops::{BinaryExpr, BinaryOp, TrinaryExpr, TrinaryOp, UnaryExpr, UnaryOp};
use radiate_core::MetricSet;
use radiate_utils::SmallStr;
use select::{MetricFlavor, MetricProperty, SelectExpr};
use std::fmt::Debug;
use std::time::Duration;

pub trait ExprQuery<I> {
    fn dispatch<'a>(&'a mut self, input: &I) -> AnyValue<'a>;
}

pub trait ExprProjection {
    fn project(&self, expr: &SelectExpr) -> Option<AnyValue<'static>>;
}

#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Literal(AnyValue<'static>),
    Selector(SelectExpr),
    Aggregate(AggExpr),
    Buffer(BufferExpr),
    Binary(BinaryExpr),
    Unary(UnaryExpr),
    Trinary(TrinaryExpr),
}

impl Expr {
    fn try_swap_metric_flavor(&mut self, to: MetricFlavor) -> bool {
        match self {
            Expr::Selector(SelectExpr::Metric(name, property, _)) => {
                *self = Expr::Selector(SelectExpr::Metric(name.clone(), property.clone(), to));
                true
            }
            _ => false,
        }
    }

    fn try_swap_metric_property(&mut self, to: MetricProperty) -> bool {
        match self {
            Expr::Selector(SelectExpr::Metric(name, _, flavor)) => {
                *self = Expr::Selector(SelectExpr::Metric(name.clone(), to, *flavor));
                true
            }
            _ => false,
        }
    }

    fn try_swap_metric_property_or(
        mut self,
        to: MetricProperty,
        func: impl FnOnce(Self) -> Expr,
    ) -> Expr {
        if self.try_swap_metric_property(to) {
            return self;
        }

        func(self)
    }

    pub fn time(mut self) -> Expr {
        self.try_swap_metric_flavor(MetricFlavor::Time);
        self
    }

    pub fn value(mut self) -> Expr {
        self.try_swap_metric_flavor(MetricFlavor::Value);
        self
    }

    pub fn rolling(self, window_size: usize) -> Expr {
        Expr::Buffer(BufferExpr::new(self, window_size))
    }

    /// Aggregates
    pub fn sum(self) -> Expr {
        self.try_swap_metric_property_or(MetricProperty::Sum, |s| {
            Expr::Aggregate(AggExpr::new(s, Rollup::Sum))
        })
    }

    pub fn mean(self) -> Expr {
        self.try_swap_metric_property_or(MetricProperty::Mean, |s| {
            Expr::Aggregate(AggExpr::new(s, Rollup::Mean))
        })
    }

    pub fn stddev(self) -> Expr {
        self.try_swap_metric_property_or(MetricProperty::StdDev, |s| {
            Expr::Aggregate(AggExpr::new(s, Rollup::StdDev))
        })
    }

    pub fn min(self) -> Expr {
        self.try_swap_metric_property_or(MetricProperty::Min, |s| {
            Expr::Aggregate(AggExpr::new(s, Rollup::Min))
        })
    }

    pub fn max(self) -> Expr {
        self.try_swap_metric_property_or(MetricProperty::Max, |s| {
            Expr::Aggregate(AggExpr::new(s, Rollup::Max))
        })
    }

    pub fn count(self) -> Expr {
        self.try_swap_metric_property_or(MetricProperty::Count, |s| {
            Expr::Aggregate(AggExpr::new(s, Rollup::Count))
        })
    }

    pub fn unique(self) -> Expr {
        Expr::Aggregate(AggExpr::new(self, Rollup::Unique))
    }

    /// Comparisons
    pub fn lt(self, rhs: impl Into<Expr>) -> Expr {
        Expr::Binary(BinaryExpr::new(self, rhs.into(), BinaryOp::Lt))
    }

    pub fn lte(self, rhs: impl Into<Expr>) -> Expr {
        Expr::Binary(BinaryExpr::new(self, rhs.into(), BinaryOp::Lte))
    }

    pub fn gt(self, rhs: impl Into<Expr>) -> Expr {
        Expr::Binary(BinaryExpr::new(self, rhs.into(), BinaryOp::Gt))
    }

    pub fn gte(self, rhs: impl Into<Expr>) -> Expr {
        Expr::Binary(BinaryExpr::new(self, rhs.into(), BinaryOp::Gte))
    }

    pub fn eq(self, rhs: impl Into<Expr>) -> Expr {
        Expr::Binary(BinaryExpr::new(self, rhs.into(), BinaryOp::Eq))
    }

    pub fn ne(self, rhs: impl Into<Expr>) -> Expr {
        Expr::Binary(BinaryExpr::new(self, rhs.into(), BinaryOp::Ne))
    }

    pub fn between(self, low: impl Into<Expr>, high: impl Into<Expr>) -> Expr {
        let low = low.into();
        let high = high.into();

        self.clone().gte(low).and(self.lte(high))
    }

    /// Logic
    pub fn and(self, rhs: impl Into<Expr>) -> Expr {
        Expr::Binary(BinaryExpr::new(self, rhs.into(), BinaryOp::And))
    }

    pub fn or(self, rhs: impl Into<Expr>) -> Expr {
        Expr::Binary(BinaryExpr::new(self, rhs.into(), BinaryOp::Or))
    }

    pub fn not(self) -> Expr {
        Expr::Unary(UnaryExpr::new(self, UnaryOp::Not))
    }

    /// Arithmetic
    pub fn neg(self) -> Expr {
        Expr::Unary(UnaryExpr::new(self, UnaryOp::Neg))
    }

    pub fn abs(self) -> Expr {
        Expr::Unary(UnaryExpr::new(self, UnaryOp::Abs))
    }

    pub fn add(self, rhs: impl Into<Expr>) -> Expr {
        Expr::Binary(BinaryExpr::new(self, rhs.into(), BinaryOp::Add))
    }

    pub fn sub(self, rhs: impl Into<Expr>) -> Expr {
        Expr::Binary(BinaryExpr::new(self, rhs.into(), BinaryOp::Sub))
    }

    pub fn mul(self, rhs: impl Into<Expr>) -> Expr {
        Expr::Binary(BinaryExpr::new(self, rhs.into(), BinaryOp::Mul))
    }

    pub fn div(self, rhs: impl Into<Expr>) -> Expr {
        Expr::Binary(BinaryExpr::new(self, rhs.into(), BinaryOp::Div))
    }

    pub fn clamp(self, min: impl Into<Expr>, max: impl Into<Expr>) -> Expr {
        Expr::Trinary(TrinaryExpr::new(
            self,
            min.into(),
            max.into(),
            TrinaryOp::Clamp,
        ))
    }
}

pub mod expr {
    use super::*;

    /// Selectors
    pub fn metric(name: impl Into<SmallStr>) -> Expr {
        Expr::Selector(SelectExpr::Metric(
            name.into(),
            MetricProperty::LastValue,
            MetricFlavor::Value,
        ))
    }

    /// Conditionals
    pub fn when(cond: impl Into<Expr>) -> When {
        When::new(cond.into())
    }
}

impl<I> ExprQuery<I> for Expr
where
    I: ExprProjection,
{
    fn dispatch<'a>(&'a mut self, input: &I) -> AnyValue<'a> {
        match self {
            Expr::Literal(value) => value.clone(),
            Expr::Selector(selector) => selector.dispatch(input),
            Expr::Aggregate(child) => child.dispatch(input),
            Expr::Buffer(child) => child.dispatch(input),
            Expr::Trinary(child) => child.dispatch(input),
            Expr::Binary(child) => child.dispatch(input),
            Expr::Unary(child) => child.dispatch(input),
        }
    }
}

impl ExprProjection for MetricSet {
    fn project(&self, expr: &SelectExpr) -> Option<AnyValue<'static>> {
        let value_to_float32 =
            |value: Option<f32>| value.map(AnyValue::Float32).unwrap_or(AnyValue::Null);

        let value_to_duration =
            |value: Option<Duration>| value.map(AnyValue::Duration).unwrap_or(AnyValue::Null);

        match expr {
            SelectExpr::Metric(name, property, flavor) => {
                self.get(name).map(|metric| match flavor {
                    MetricFlavor::Value => match property {
                        MetricProperty::LastValue => AnyValue::Float32(metric.last_value()),
                        MetricProperty::Mean => value_to_float32(metric.value_mean()),
                        MetricProperty::StdDev => value_to_float32(metric.value_std_dev()),
                        MetricProperty::Min => value_to_float32(metric.value_min()),
                        MetricProperty::Max => value_to_float32(metric.value_max()),
                        MetricProperty::Sum => value_to_float32(metric.value_sum()),
                        MetricProperty::Count => AnyValue::UInt64(metric.count() as u64),
                        MetricProperty::Version => AnyValue::UInt64(metric.version()),
                        MetricProperty::UpdateCount => {
                            AnyValue::UInt64(metric.update_count() as u64)
                        }
                    },
                    MetricFlavor::Time => match property {
                        MetricProperty::LastValue => AnyValue::Duration(metric.last_time()),
                        MetricProperty::Mean => value_to_duration(metric.time_mean()),
                        MetricProperty::StdDev => value_to_duration(metric.time_std_dev()),
                        MetricProperty::Min => value_to_duration(metric.time_min()),
                        MetricProperty::Max => value_to_duration(metric.time_max()),
                        MetricProperty::Sum => value_to_duration(metric.time_sum()),
                        MetricProperty::Count => AnyValue::UInt64(metric.count() as u64),
                        MetricProperty::Version => AnyValue::UInt64(metric.version()),
                        MetricProperty::UpdateCount => {
                            AnyValue::UInt64(metric.update_count() as u64)
                        }
                    },
                })
            }
            _ => None,
        }
    }
}

impl From<f32> for AnyValue<'_> {
    fn from(value: f32) -> Self {
        AnyValue::Float32(value)
    }
}

impl From<Duration> for AnyValue<'_> {
    fn from(value: Duration) -> Self {
        AnyValue::Duration(value)
    }
}

impl From<f32> for Expr {
    fn from(value: f32) -> Self {
        Expr::Literal(AnyValue::Float32(value))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn f32_of(value: AnyValue<'_>) -> f32 {
        value.extract::<f32>().unwrap()
    }

    fn bool_of(value: AnyValue<'_>) -> bool {
        if let AnyValue::Bool(b) = value {
            b
        } else {
            false
        }
    }

    fn u64_of(value: AnyValue<'_>) -> u64 {
        value.extract::<u64>().unwrap()
    }

    #[test]
    fn test_metric_selector_returns_last_value() {
        let mut expr = expr::metric("accuracy");
        let mut metrics = MetricSet::default();

        metrics.upsert(("accuracy", 42.0));
        let result = expr.dispatch(&metrics);

        assert_eq!(f32_of(result), 42.0);
    }

    #[test]
    fn test_missing_metric_returns_null() {
        let mut expr = expr::metric("does_not_exist");
        let metrics = MetricSet::default();

        let result = expr.dispatch(&metrics);

        assert_eq!(result, AnyValue::Null);
    }

    #[test]
    fn test_rolling_returns_slice_with_expected_window_contents() {
        let mut expr = expr::metric("accuracy").rolling(3);
        let mut metrics = MetricSet::default();

        metrics.upsert(("accuracy", 1.0));
        let result = expr.dispatch(&metrics);
        assert!(result.is_nested());
        if let AnyValue::Slice(values) = result {
            assert_eq!(values.len(), 1);
            assert_eq!(values[0].clone().extract::<f32>().unwrap(), 1.0);
        }

        metrics.upsert(("accuracy", 2.0));
        let result = expr.dispatch(&metrics);
        assert!(result.is_nested());
        if let AnyValue::Slice(values) = result {
            assert_eq!(values.len(), 2);
            assert_eq!(values[0].clone().extract::<f32>().unwrap(), 1.0);
            assert_eq!(values[1].clone().extract::<f32>().unwrap(), 2.0);
        }

        metrics.upsert(("accuracy", 3.0));
        let result = expr.dispatch(&metrics);

        assert!(result.is_nested());

        if let AnyValue::Slice(values) = result {
            assert_eq!(values.len(), 3);
            assert_eq!(values[0].clone().extract::<f32>().unwrap(), 1.0);
            assert_eq!(values[1].clone().extract::<f32>().unwrap(), 2.0);
            assert_eq!(values[2].clone().extract::<f32>().unwrap(), 3.0);
        }

        metrics.upsert(("accuracy", 4.0));
        let result = expr.dispatch(&metrics);
        assert!(result.is_nested());

        if let AnyValue::Slice(values) = result {
            assert_eq!(values.len(), 3);
            assert_eq!(values[0].clone().extract::<f32>().unwrap(), 2.0);
            assert_eq!(values[1].clone().extract::<f32>().unwrap(), 3.0);
            assert_eq!(values[2].clone().extract::<f32>().unwrap(), 4.0);
        }
    }

    #[test]
    fn test_rolling_mean() {
        let mut expr = expr::metric("accuracy").rolling(3).mean();
        let mut metrics = MetricSet::default();

        metrics.upsert(("accuracy", 1.0));
        assert!((f32_of(expr.dispatch(&metrics)) - 1.0).abs() < 1e-6);

        metrics.upsert(("accuracy", 2.0));
        assert!((f32_of(expr.dispatch(&metrics)) - 1.5).abs() < 1e-6);

        metrics.upsert(("accuracy", 3.0));
        assert!((f32_of(expr.dispatch(&metrics)) - 2.0).abs() < 1e-6);

        metrics.upsert(("accuracy", 4.0));
        assert!((f32_of(expr.dispatch(&metrics)) - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_rolling_sum() {
        let mut expr = expr::metric("accuracy").rolling(3).sum();
        let mut metrics = MetricSet::default();

        metrics.upsert(("accuracy", 1.0));
        assert!((f32_of(expr.dispatch(&metrics)) - 1.0).abs() < 1e-6);

        metrics.upsert(("accuracy", 2.0));
        assert!((f32_of(expr.dispatch(&metrics)) - 3.0).abs() < 1e-6);

        metrics.upsert(("accuracy", 3.0));
        assert!((f32_of(expr.dispatch(&metrics)) - 6.0).abs() < 1e-6);

        metrics.upsert(("accuracy", 4.0));
        assert!((f32_of(expr.dispatch(&metrics)) - 9.0).abs() < 1e-6);
    }

    #[test]
    fn test_rolling_min_and_max() {
        let mut min_expr = expr::metric("accuracy").rolling(4).min();
        let mut max_expr = expr::metric("accuracy").rolling(4).max();
        let mut metrics = MetricSet::default();

        for value in [3.0, 1.0, 4.0, 2.0] {
            metrics.upsert(("accuracy", value));
            min_expr.dispatch(&metrics);
            max_expr.dispatch(&metrics);
        }

        assert!((f32_of(min_expr.dispatch(&metrics)) - 1.0).abs() < 1e-6);
        assert!((f32_of(max_expr.dispatch(&metrics)) - 4.0).abs() < 1e-6);
    }

    #[test]
    fn test_rolling_count() {
        let mut expr = expr::metric("accuracy").rolling(3).count();
        let mut metrics = MetricSet::default();

        metrics.upsert(("accuracy", 10.0));
        assert_eq!(u64_of(expr.dispatch(&metrics)), 1);

        metrics.upsert(("accuracy", 11.0));
        assert_eq!(u64_of(expr.dispatch(&metrics)), 2);

        metrics.upsert(("accuracy", 12.0));
        assert_eq!(u64_of(expr.dispatch(&metrics)), 3);

        metrics.upsert(("accuracy", 13.0));
        assert_eq!(u64_of(expr.dispatch(&metrics)), 3);
    }

    #[test]
    fn test_rolling_n_unique() {
        let mut expr = expr::metric("accuracy").rolling(5).unique().count();
        let mut metrics = MetricSet::default();

        metrics.upsert(("accuracy", 1.0));
        assert_eq!(u64_of(expr.dispatch(&metrics)), 1);

        metrics.upsert(("accuracy", 2.0));
        assert_eq!(u64_of(expr.dispatch(&metrics)), 2);

        metrics.upsert(("accuracy", 2.0));
        assert_eq!(u64_of(expr.dispatch(&metrics)), 2);

        metrics.upsert(("accuracy", 3.0));
        assert_eq!(u64_of(expr.dispatch(&metrics)), 3);

        metrics.upsert(("accuracy", 1.0));
        assert_eq!(u64_of(expr.dispatch(&metrics)), 3);
    }

    #[test]
    fn test_lt_comparison_true_and_false() {
        let mut expr = expr::metric("accuracy").lt(expr::metric("loss"));
        let mut metrics = MetricSet::default();

        metrics.upsert(("accuracy", 0.8));
        metrics.upsert(("loss", 1.2));
        assert_eq!(bool_of(expr.dispatch(&metrics)), true);

        metrics.upsert(("accuracy", 2.0));
        metrics.upsert(("loss", 1.2));
        assert_eq!(bool_of(expr.dispatch(&metrics)), false);
    }

    #[test]
    fn test_gte_comparison() {
        let mut expr = expr::metric("accuracy").gte(expr::metric("target"));
        let mut metrics = MetricSet::default();

        metrics.upsert(("accuracy", 0.95));
        metrics.upsert(("target", 0.90));
        assert!(bool_of(expr.dispatch(&metrics)));

        metrics.upsert(("accuracy", 0.85));
        metrics.upsert(("target", 0.90));
        assert!(!bool_of(expr.dispatch(&metrics)));
    }

    #[test]
    fn test_eq_comparison_uses_epsilon() {
        let mut expr = expr::metric("a").eq(expr::metric("b"));
        let mut metrics = MetricSet::default();

        metrics.upsert(("a", 1.0f32));
        metrics.upsert(("b", 1.0f32));
        assert!(bool_of(expr.dispatch(&metrics)));
    }

    #[test]
    fn test_ne_comparison() {
        let mut expr = expr::metric("a").ne(expr::metric("b"));
        let mut metrics = MetricSet::default();

        metrics.upsert(("a", 1.0f32));
        metrics.upsert(("b", 2.0f32));
        assert!(bool_of(expr.dispatch(&metrics)));

        metrics.upsert(("a", 5.0f32));
        metrics.upsert(("b", 5.0f32));
        assert!(!bool_of(expr.dispatch(&metrics)));
    }

    #[test]
    fn test_metric_projection_uses_metricset_property_mean() {
        let expr = SelectExpr::Metric("accuracy".into(), MetricProperty::Mean, MetricFlavor::Value);
        let mut metrics = MetricSet::default();

        metrics.upsert(("accuracy", 1.0));
        metrics.upsert(("accuracy", 2.0));
        metrics.upsert(("accuracy", 3.0));

        let result = metrics.project(&expr).unwrap();
        assert!((f32_of(result) - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_metric_projection_uses_metricset_property_count() {
        let expr = SelectExpr::Metric(
            "accuracy".into(),
            MetricProperty::Count,
            MetricFlavor::Value,
        );
        let mut metrics = MetricSet::default();

        metrics.upsert(("accuracy", 1.0));
        metrics.upsert(("accuracy", 2.0));
        metrics.upsert(("accuracy", 3.0));

        let result = metrics.project(&expr).unwrap();
        assert_eq!(u64_of(result), 3);
    }

    #[test]
    fn test_between_inclusive() {
        let mut expr = expr::metric("x").between(1.0, 3.0);
        let mut metrics = MetricSet::default();

        metrics.upsert(("x", 1.0));
        assert!(bool_of(expr.dispatch(&metrics)));

        metrics.upsert(("x", 2.0));
        assert!(bool_of(expr.dispatch(&metrics)));

        metrics.upsert(("x", 3.0));
        assert!(bool_of(expr.dispatch(&metrics)));

        metrics.upsert(("x", 0.99));
        assert!(!bool_of(expr.dispatch(&metrics)));

        metrics.upsert(("x", 3.01));
        assert!(!bool_of(expr.dispatch(&metrics)));
    }

    #[test]
    fn test_add_expr() {
        let mut expr = expr::metric("a").add(expr::metric("b"));
        let mut metrics = MetricSet::default();

        metrics.upsert(("a", 2.0));
        metrics.upsert(("b", 3.5));

        assert!((f32_of(expr.dispatch(&metrics)) - 5.5).abs() < 1e-6);
    }

    #[test]
    fn test_sub_expr() {
        let mut expr = expr::metric("a").sub(expr::metric("b"));
        let mut metrics = MetricSet::default();

        metrics.upsert(("a", 5.0));
        metrics.upsert(("b", 1.5));

        assert!((f32_of(expr.dispatch(&metrics)) - 3.5).abs() < 1e-6);
    }

    #[test]
    fn test_mul_expr() {
        let mut expr = expr::metric("a").mul(2.5);
        let mut metrics = MetricSet::default();

        metrics.upsert(("a", 4.0));

        assert!((f32_of(expr.dispatch(&metrics)) - 10.0).abs() < 1e-6);
    }

    #[test]
    fn test_div_expr() {
        let mut expr = expr::metric("a").div(expr::metric("b"));
        let mut metrics = MetricSet::default();

        metrics.upsert(("a", 9.0));
        metrics.upsert(("b", 3.0));

        assert!((f32_of(expr.dispatch(&metrics)) - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_div_by_zero_returns_null() {
        let mut expr = expr::metric("a").div(expr::metric("b"));
        let mut metrics = MetricSet::default();

        metrics.upsert(("a", 9.0));
        metrics.upsert(("b", 0.0));

        assert_eq!(expr.dispatch(&metrics), AnyValue::Null);
    }

    #[test]
    fn test_neg_expr() {
        let mut expr = expr::metric("a").neg();
        let mut metrics = MetricSet::default();

        metrics.upsert(("a", 4.0));

        assert!((f32_of(expr.dispatch(&metrics)) + 4.0).abs() < 1e-6);
    }

    #[test]
    fn test_abs_expr() {
        let mut expr = expr::metric("a").sub(10.0).abs();
        let mut metrics = MetricSet::default();

        metrics.upsert(("a", 4.0));
        assert!((f32_of(expr.dispatch(&metrics)) - 6.0).abs() < 1e-6);

        metrics.upsert(("a", 14.0));
        assert!((f32_of(expr.dispatch(&metrics)) - 4.0).abs() < 1e-6);
    }

    #[test]
    fn test_clamp_expr() {
        let mut expr = expr::metric("a").clamp(0.1, 0.5);
        let mut metrics = MetricSet::default();

        metrics.upsert(("a", 0.05));
        assert!((f32_of(expr.dispatch(&metrics)) - 0.1).abs() < 1e-6);

        metrics.upsert(("a", 0.25));
        assert!((f32_of(expr.dispatch(&metrics)) - 0.25).abs() < 1e-6);

        metrics.upsert(("a", 0.9));
        assert!((f32_of(expr.dispatch(&metrics)) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_duration_expr() {
        let mut expr = expr::metric("time").time().rolling(10).min();
        let mut metrics = MetricSet::default();

        metrics.upsert(("time", Duration::from_secs(5)));
        expr.dispatch(&metrics);
        metrics.upsert(("time", Duration::from_secs(3)));
        expr.dispatch(&metrics);
        metrics.upsert(("time", Duration::from_secs(8)));
        let result = expr.dispatch(&metrics);

        assert_eq!(result, AnyValue::Duration(Duration::from_secs(3)));
    }
}
