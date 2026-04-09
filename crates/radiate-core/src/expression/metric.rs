use crate::{
    AggregateExprNode, CompareExprNode, CompareOp, ExprEval, ExprNode, MetricSet, Rollup, Value,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MetricValueKind {
    Last,
    Mean,
    StdDev,
    Variance,
    Min,
    Max,
    Sum,
    Count,
}

#[derive(Clone)]
pub struct MetricExprNode {
    name: &'static str,
    kind: MetricValueKind,
}

impl MetricExprNode {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            kind: MetricValueKind::Last,
        }
    }

    pub fn last(mut self) -> Self {
        self.kind = MetricValueKind::Last;
        self
    }

    pub fn mean(mut self) -> Self {
        self.kind = MetricValueKind::Mean;
        self
    }

    pub fn stddev(mut self) -> Self {
        self.kind = MetricValueKind::StdDev;
        self
    }

    pub fn variance(mut self) -> Self {
        self.kind = MetricValueKind::Variance;
        self
    }

    pub fn min(mut self) -> Self {
        self.kind = MetricValueKind::Min;
        self
    }

    pub fn max(mut self) -> Self {
        self.kind = MetricValueKind::Max;
        self
    }

    pub fn sum(mut self) -> Self {
        self.kind = MetricValueKind::Sum;
        self
    }

    pub fn count(mut self) -> Self {
        self.kind = MetricValueKind::Count;
        self
    }

    pub fn stagnant<'a>(self, limit: usize) -> ExprNode<'a> {
        ExprNode::Aggregate(AggregateExprNode::new(self.into(), Rollup::NUnique, limit)).lte(1)
    }

    pub fn rolling<'a>(self, window_size: usize) -> ExprNode<'a> {
        ExprNode::Aggregate(AggregateExprNode::new(
            self.into(),
            Rollup::Mean,
            window_size,
        ))
    }

    pub fn lt<'a>(self, rhs: impl Into<ExprNode<'a>>) -> ExprNode<'a> {
        ExprNode::Compare(CompareExprNode::new(self.into(), rhs.into(), CompareOp::Lt))
    }

    pub fn lte<'a>(self, rhs: impl Into<ExprNode<'a>>) -> ExprNode<'a> {
        ExprNode::Compare(CompareExprNode::new(
            self.into(),
            rhs.into(),
            CompareOp::Lte,
        ))
    }

    pub fn gt<'a>(self, rhs: impl Into<ExprNode<'a>>) -> ExprNode<'a> {
        ExprNode::Compare(CompareExprNode::new(self.into(), rhs.into(), CompareOp::Gt))
    }

    pub fn gte<'a>(self, rhs: impl Into<ExprNode<'a>>) -> ExprNode<'a> {
        ExprNode::Compare(CompareExprNode::new(
            self.into(),
            rhs.into(),
            CompareOp::Gte,
        ))
    }

    pub fn eq<'a>(self, rhs: impl Into<ExprNode<'a>>) -> ExprNode<'a> {
        ExprNode::Compare(CompareExprNode::new(self.into(), rhs.into(), CompareOp::Eq))
    }
}

impl ExprEval<MetricSet> for MetricExprNode {
    fn eval<'a>(&'a mut self, metrics: &'a MetricSet) -> Value<'a> {
        if let Some(metric) = metrics.get(&self.name) {
            let value = match self.kind {
                MetricValueKind::Last => metric.last_value(),
                MetricValueKind::Mean => metric.value_mean().unwrap_or(0.0),
                MetricValueKind::StdDev => metric.value_std_dev().unwrap_or(0.0),
                MetricValueKind::Variance => metric.value_variance().unwrap_or(0.0),
                MetricValueKind::Min => metric.value_min().unwrap_or(0.0),
                MetricValueKind::Max => metric.value_max().unwrap_or(0.0),
                MetricValueKind::Sum => metric.value_sum().unwrap_or(0.0),
                MetricValueKind::Count => metric.count() as f32,
            };

            let value = Value::Float32(value);

            value
        } else {
            Value::Null
        }
    }
}
