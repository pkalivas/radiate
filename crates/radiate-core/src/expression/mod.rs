mod scalar;

use crate::{MetricSet, Statistic};
use radiate_utils::{SmallStr, WindowBuffer};
use std::collections::HashSet;
use std::fmt::Debug;
use std::sync::Arc;
use std::time::Duration;

pub use scalar::ExprValue;

pub trait ExprQuery<I> {
    fn dispatch<'a>(&'a mut self, input: &I) -> ExprValue<'a>;
}

pub trait ExprProjection {
    fn project(&self, expr: &SelectExpr) -> Option<ExprValue<'static>>;
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Rollup {
    Mean,
    StdDev,
    Min,
    Max,
    Sum,
    Count,
    Unique,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ComparisonOp {
    Lt,
    Lte,
    Gt,
    Gte,
    Eq,
    Ne,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MetricProperty {
    LastValue,
    Mean,
    StdDev,
    Min,
    Max,
    Sum,
    Count,
    Version,
    UpdateCount,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ArithmeticOp {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UnaryOp {
    Not,
    Neg,
    Abs,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TrinaryOp {
    If,
    Clamp,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MetricFlavor {
    Value,
    Time,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SelectExpr {
    Metric(SmallStr, MetricProperty, MetricFlavor),
    Nth(usize),
}

impl<T> ExprQuery<T> for SelectExpr
where
    T: ExprProjection,
{
    fn dispatch<'a>(&'a mut self, input: &T) -> ExprValue<'a> {
        input.project(self).unwrap_or(ExprValue::Null)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AggExpr {
    child: Arc<Expr>,
    rollup: Rollup,
}

impl AggExpr {
    pub fn new(child: Expr, rollup: Rollup) -> Self {
        Self {
            child: Arc::new(child),
            rollup,
        }
    }

    fn compute_rollup<'a>(values: &[ExprValue], rollup: Rollup) -> ExprValue<'a> {
        let mut stats = Statistic::default();
        for value in values.iter() {
            if let Some(v) = value.clone().extract::<f32>() {
                stats.add(v);
            }
        }

        match rollup {
            Rollup::Mean => ExprValue::Float32(stats.mean()),
            Rollup::StdDev => ExprValue::Float32(stats.std_dev()),
            Rollup::Min => ExprValue::Float32(stats.min()),
            Rollup::Max => ExprValue::Float32(stats.max()),
            Rollup::Sum => ExprValue::Float32(stats.sum()),
            Rollup::Count => ExprValue::UInt64(stats.count() as u64),
            Rollup::Unique => ExprValue::Null,
        }
    }
}

impl<T> ExprQuery<T> for AggExpr
where
    T: ExprProjection,
{
    fn dispatch<'a>(&'a mut self, input: &T) -> ExprValue<'a> {
        if let Rollup::Unique = self.rollup {
            let child_output = Arc::make_mut(&mut self.child).dispatch(input);
            return match child_output {
                ExprValue::Slice(values) => {
                    let deduped = values.iter().fold(HashSet::new(), |mut acc, v| {
                        acc.insert(v.clone());
                        acc
                    });

                    return ExprValue::Vector(deduped.into_iter().collect());
                }
                ExprValue::Vector(values) => {
                    let deduped = values.into_iter().fold(HashSet::new(), |mut acc, v| {
                        acc.insert(v);
                        acc
                    });

                    ExprValue::Vector(deduped.into_iter().collect())
                }
                _ => ExprValue::Null,
            };
        }
        let child_output = Arc::make_mut(&mut self.child).dispatch(input);

        match child_output {
            ExprValue::Slice(values) => Self::compute_rollup(values, self.rollup),
            ExprValue::Vector(values) => Self::compute_rollup(&values, self.rollup),
            _ => child_output,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct BufferExpr {
    buffer: WindowBuffer<ExprValue<'static>>,
    child: Arc<Expr>,
}

impl BufferExpr {
    pub fn new(child: Expr, window_size: usize) -> Self {
        Self {
            buffer: WindowBuffer::with_window(window_size),
            child: Arc::new(child),
        }
    }
}

impl<T> ExprQuery<T> for BufferExpr
where
    T: ExprProjection,
{
    fn dispatch<'a>(&'a mut self, input: &T) -> ExprValue<'a> {
        let child_output = Arc::make_mut(&mut self.child).dispatch(input).into_static();
        self.buffer.push(child_output);
        ExprValue::Slice(&self.buffer.values())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CompareExpr {
    lhs: Arc<Expr>,
    rhs: Arc<Expr>,
    op: ComparisonOp,
}

impl CompareExpr {
    pub fn new(lhs: Expr, rhs: Expr, op: ComparisonOp) -> Self {
        Self {
            lhs: Arc::new(lhs),
            rhs: Arc::new(rhs),
            op,
        }
    }
}

impl<T> ExprQuery<T> for CompareExpr
where
    T: ExprProjection,
{
    fn dispatch<'a>(&'a mut self, metrics: &T) -> ExprValue<'a> {
        let lhs = Arc::make_mut(&mut self.lhs)
            .dispatch(metrics)
            .extract::<f32>();
        let rhs = Arc::make_mut(&mut self.rhs)
            .dispatch(metrics)
            .extract::<f32>();

        match (lhs, rhs) {
            (Some(lhs), Some(rhs)) => {
                let result = match self.op {
                    ComparisonOp::Lt => lhs < rhs,
                    ComparisonOp::Lte => lhs <= rhs,
                    ComparisonOp::Gt => lhs > rhs,
                    ComparisonOp::Gte => lhs >= rhs,
                    ComparisonOp::Eq => (lhs - rhs).abs() <= f32::EPSILON,
                    ComparisonOp::Ne => (lhs - rhs).abs() > f32::EPSILON,
                };

                ExprValue::Bool(result)
            }
            _ => ExprValue::Null,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LogicOp {
    And,
    Or,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LogicExpr {
    lhs: Arc<Expr>,
    rhs: Arc<Expr>,
    op: LogicOp,
}

impl LogicExpr {
    pub fn new(lhs: Expr, rhs: Expr, op: LogicOp) -> Self {
        Self {
            lhs: Arc::new(lhs),
            rhs: Arc::new(rhs),
            op,
        }
    }
}

impl<T> ExprQuery<T> for LogicExpr
where
    T: ExprProjection,
{
    fn dispatch<'a>(&'a mut self, metrics: &T) -> ExprValue<'a> {
        match self.op {
            LogicOp::And => {
                let lhs = Arc::make_mut(&mut self.lhs).dispatch(metrics).as_bool();

                if !lhs {
                    return ExprValue::Bool(false);
                }
                ExprValue::Bool(Arc::make_mut(&mut self.rhs).dispatch(metrics).as_bool())
            }
            LogicOp::Or => {
                let lhs = Arc::make_mut(&mut self.lhs).dispatch(metrics).as_bool();
                if lhs {
                    return ExprValue::Bool(true);
                }
                ExprValue::Bool(Arc::make_mut(&mut self.rhs).dispatch(metrics).as_bool())
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TrinaryExpr {
    first: Arc<Expr>,
    second: Arc<Expr>,
    third: Arc<Expr>,
    operation: TrinaryOp,
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
    fn dispatch<'a>(&'a mut self, input: &T) -> ExprValue<'a> {
        match self.operation {
            TrinaryOp::If => {
                let condition = Arc::make_mut(&mut self.first).dispatch(input).as_bool();
                if condition {
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
                    (Some(value), Some(min), Some(max)) => {
                        ExprValue::Float32(value.clamp(min, max))
                    }
                    _ => ExprValue::Null,
                }
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct BinaryExpr {
    lhs: Arc<Expr>,
    rhs: Arc<Expr>,
    op: ArithmeticOp,
}

impl BinaryExpr {
    pub fn new(lhs: Expr, rhs: Expr, op: ArithmeticOp) -> Self {
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
    fn dispatch<'a>(&'a mut self, input: &T) -> ExprValue<'a> {
        let lhs = Arc::make_mut(&mut self.lhs)
            .dispatch(input)
            .extract::<f32>();
        let rhs = Arc::make_mut(&mut self.rhs)
            .dispatch(input)
            .extract::<f32>();

        match (lhs, rhs) {
            (Some(lhs), Some(rhs)) => {
                let value = match self.op {
                    ArithmeticOp::Add => lhs + rhs,
                    ArithmeticOp::Sub => lhs - rhs,
                    ArithmeticOp::Mul => lhs * rhs,
                    ArithmeticOp::Div => {
                        if rhs.abs() <= f32::EPSILON {
                            return ExprValue::Null;
                        }
                        lhs / rhs
                    }
                };

                ExprValue::Float32(value)
            }
            _ => ExprValue::Null,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct UnaryExpr {
    child: Arc<Expr>,
    op: UnaryOp,
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
    fn dispatch<'a>(&'a mut self, input: &T) -> ExprValue<'a> {
        let value = Arc::make_mut(&mut self.child).dispatch(input);

        match self.op {
            UnaryOp::Not => ExprValue::Bool(!value.as_bool()),
            UnaryOp::Neg => match value.extract::<f32>() {
                Some(v) => ExprValue::Float32(-v),
                None => ExprValue::Null,
            },
            UnaryOp::Abs => match value.extract::<f32>() {
                Some(v) => ExprValue::Float32(v.abs()),
                None => ExprValue::Null,
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ClampExpr {
    value: Arc<Expr>,
    min: Arc<Expr>,
    max: Arc<Expr>,
}

impl ClampExpr {
    pub fn new(value: Expr, min: Expr, max: Expr) -> Self {
        Self {
            value: Arc::new(value),
            min: Arc::new(min),
            max: Arc::new(max),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct When {
    cond: Arc<Expr>,
}

impl When {
    pub fn new(cond: impl Into<Expr>) -> Self {
        Self {
            cond: Arc::new(cond.into()),
        }
    }

    pub fn then(self, then_expr: impl Into<Expr>) -> Then {
        Then {
            cond: self.cond,
            then_expr: Arc::new(then_expr.into()),
        }
    }
}

pub struct Then {
    cond: Arc<Expr>,
    then_expr: Arc<Expr>,
}

impl Then {
    pub fn otherwise(self, else_expr: impl Into<Expr>) -> Expr {
        let cond = Arc::try_unwrap(self.cond).unwrap_or_else(|val| (*val).clone());
        let then_expr = Arc::try_unwrap(self.then_expr).unwrap_or_else(|val| (*val).clone());

        Expr::Trinary(TrinaryExpr::new(
            cond,
            then_expr,
            else_expr.into(),
            TrinaryOp::If,
        ))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Empty,
    Literal(ExprValue<'static>),
    Selector(SelectExpr),
    Aggregate(AggExpr),
    Buffer(BufferExpr),
    Compare(CompareExpr),
    Logic(LogicExpr),
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

    pub fn time(mut self) -> Expr {
        if !self.try_swap_metric_flavor(MetricFlavor::Time) {
            self
        } else {
            self
        }
    }

    pub fn value(mut self) -> Expr {
        if !self.try_swap_metric_flavor(MetricFlavor::Value) {
            self
        } else {
            self
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
        Expr::Compare(CompareExpr::new(self, rhs.into(), ComparisonOp::Lt))
    }

    pub fn lte(self, rhs: impl Into<Expr>) -> Expr {
        Expr::Compare(CompareExpr::new(self, rhs.into(), ComparisonOp::Lte))
    }

    pub fn gt(self, rhs: impl Into<Expr>) -> Expr {
        Expr::Compare(CompareExpr::new(self, rhs.into(), ComparisonOp::Gt))
    }

    pub fn gte(self, rhs: impl Into<Expr>) -> Expr {
        Expr::Compare(CompareExpr::new(self, rhs.into(), ComparisonOp::Gte))
    }

    pub fn eq(self, rhs: impl Into<Expr>) -> Expr {
        Expr::Compare(CompareExpr::new(self, rhs.into(), ComparisonOp::Eq))
    }

    pub fn ne(self, rhs: impl Into<Expr>) -> Expr {
        Expr::Compare(CompareExpr::new(self, rhs.into(), ComparisonOp::Ne))
    }

    pub fn between(self, low: impl Into<Expr>, high: impl Into<Expr>) -> Expr {
        let low = low.into();
        let high = high.into();

        self.clone().gte(low).and(self.lte(high))
    }

    /// Logic
    pub fn and(self, rhs: impl Into<Expr>) -> Expr {
        Expr::Logic(LogicExpr::new(self, rhs.into(), LogicOp::And))
    }

    pub fn or(self, rhs: impl Into<Expr>) -> Expr {
        Expr::Logic(LogicExpr::new(self, rhs.into(), LogicOp::Or))
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
        Expr::Binary(BinaryExpr::new(self, rhs.into(), ArithmeticOp::Add))
    }

    pub fn sub(self, rhs: impl Into<Expr>) -> Expr {
        Expr::Binary(BinaryExpr::new(self, rhs.into(), ArithmeticOp::Sub))
    }

    pub fn mul(self, rhs: impl Into<Expr>) -> Expr {
        Expr::Binary(BinaryExpr::new(self, rhs.into(), ArithmeticOp::Mul))
    }

    pub fn div(self, rhs: impl Into<Expr>) -> Expr {
        Expr::Binary(BinaryExpr::new(self, rhs.into(), ArithmeticOp::Div))
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
    fn dispatch<'a>(&'a mut self, input: &I) -> ExprValue<'a> {
        match self {
            Expr::Literal(value) => value.clone(),
            Expr::Selector(selector) => selector.dispatch(input),
            Expr::Aggregate(child) => child.dispatch(input),
            Expr::Buffer(child) => child.dispatch(input),
            Expr::Compare(child) => child.dispatch(input),
            Expr::Logic(child) => child.dispatch(input),
            Expr::Trinary(child) => child.dispatch(input),
            Expr::Binary(child) => child.dispatch(input),
            Expr::Unary(child) => child.dispatch(input),
            Expr::Empty => ExprValue::Null,
        }
    }
}

impl ExprProjection for MetricSet {
    fn project(&self, expr: &SelectExpr) -> Option<ExprValue<'static>> {
        let value_to_float32 =
            |value: Option<f32>| value.map(ExprValue::Float32).unwrap_or(ExprValue::Null);

        let value_to_duration =
            |value: Option<Duration>| value.map(ExprValue::Duration).unwrap_or(ExprValue::Null);

        match expr {
            SelectExpr::Metric(name, property, flavor) => {
                self.get(name).map(|metric| match flavor {
                    MetricFlavor::Value => match property {
                        MetricProperty::LastValue => ExprValue::Float32(metric.last_value()),
                        MetricProperty::Mean => value_to_float32(metric.value_mean()),
                        MetricProperty::StdDev => value_to_float32(metric.value_std_dev()),
                        MetricProperty::Min => value_to_float32(metric.value_min()),
                        MetricProperty::Max => value_to_float32(metric.value_max()),
                        MetricProperty::Sum => value_to_float32(metric.value_sum()),
                        MetricProperty::Count => ExprValue::UInt64(metric.count() as u64),
                        MetricProperty::Version => ExprValue::UInt64(metric.version()),
                        _ => ExprValue::Null,
                    },
                    MetricFlavor::Time => match property {
                        MetricProperty::LastValue => ExprValue::Duration(metric.last_time()),
                        MetricProperty::Mean => value_to_duration(metric.time_mean()),
                        MetricProperty::StdDev => value_to_duration(metric.time_std_dev()),
                        MetricProperty::Min => value_to_duration(metric.time_min()),
                        MetricProperty::Max => value_to_duration(metric.time_max()),
                        MetricProperty::Sum => value_to_duration(metric.time_sum()),
                        MetricProperty::Count => ExprValue::UInt64(metric.count() as u64),
                        MetricProperty::Version => ExprValue::UInt64(metric.version()),
                        _ => ExprValue::Null,
                    },
                })
            }
            _ => None,
        }
    }
}

impl From<f32> for ExprValue<'_> {
    fn from(value: f32) -> Self {
        ExprValue::Float32(value)
    }
}

impl From<Duration> for ExprValue<'_> {
    fn from(value: Duration) -> Self {
        ExprValue::Duration(value)
    }
}

impl From<f32> for Expr {
    fn from(value: f32) -> Self {
        Expr::Literal(ExprValue::Float32(value))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn f32_of(value: ExprValue<'_>) -> f32 {
        value.extract::<f32>().unwrap()
    }

    fn bool_of(value: ExprValue<'_>) -> bool {
        if let ExprValue::Bool(b) = value {
            b
        } else {
            false
        }
    }

    fn u64_of(value: ExprValue<'_>) -> u64 {
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

        assert_eq!(result, ExprValue::Null);
    }

    #[test]
    fn test_rolling_returns_slice_with_expected_window_contents() {
        let mut expr = expr::metric("accuracy").rolling(3);
        let mut metrics = MetricSet::default();

        metrics.upsert(("accuracy", 1.0));
        let result = expr.dispatch(&metrics);
        assert!(result.is_nested());
        if let ExprValue::Slice(values) = result {
            assert_eq!(values.len(), 1);
            assert_eq!(values[0].clone().extract::<f32>().unwrap(), 1.0);
        }

        metrics.upsert(("accuracy", 2.0));
        let result = expr.dispatch(&metrics);
        assert!(result.is_nested());
        if let ExprValue::Slice(values) = result {
            assert_eq!(values.len(), 2);
            assert_eq!(values[0].clone().extract::<f32>().unwrap(), 1.0);
            assert_eq!(values[1].clone().extract::<f32>().unwrap(), 2.0);
        }

        metrics.upsert(("accuracy", 3.0));
        let result = expr.dispatch(&metrics);

        assert!(result.is_nested());

        if let ExprValue::Slice(values) = result {
            assert_eq!(values.len(), 3);
            assert_eq!(values[0].clone().extract::<f32>().unwrap(), 1.0);
            assert_eq!(values[1].clone().extract::<f32>().unwrap(), 2.0);
            assert_eq!(values[2].clone().extract::<f32>().unwrap(), 3.0);
        }

        metrics.upsert(("accuracy", 4.0));
        let result = expr.dispatch(&metrics);
        assert!(result.is_nested());

        if let ExprValue::Slice(values) = result {
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

        assert_eq!(expr.dispatch(&metrics), ExprValue::Null);
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
}
