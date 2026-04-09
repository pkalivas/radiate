use crate::{CompareExprNode, CompareOp, ExprEval, ExprNode, MetricSet, Statistic, Value};
use radiate_utils::WindowBuffer;

#[derive(Clone)]
pub struct SequenceExprNode<'a> {
    buffer: WindowBuffer<Value<'a>>,
    child: Box<ExprNode<'a>>,
}

impl<'a> SequenceExprNode<'a> {
    pub fn new(child: ExprNode<'a>, window_size: usize) -> Self {
        Self {
            buffer: WindowBuffer::with_window(window_size),
            child: Box::new(child),
        }
    }
}

impl ExprEval<MetricSet> for SequenceExprNode<'_> {
    fn eval<'a>(&'a mut self, metrics: &'a MetricSet) -> Value<'a> {
        let child_value = self.child.eval(metrics).into_static();

        self.buffer.push(child_value);

        Value::Slice(&self.buffer.values())
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Rollup {
    Mean,
    StdDev,
    Min,
    Max,
    Sum,
    Count,
    NUnique,
}

#[derive(Clone)]
pub struct AggregateExprNode<'a> {
    child: SequenceExprNode<'a>,
    rollup: Rollup,
}

impl<'a> AggregateExprNode<'a> {
    pub fn new(child: ExprNode<'a>, rollup: Rollup, window_size: usize) -> Self {
        Self {
            child: SequenceExprNode::new(child, window_size),
            rollup,
        }
    }

    pub fn mean(mut self) -> ExprNode<'a> {
        self.rollup = Rollup::Mean;
        ExprNode::Aggregate(self)
    }

    pub fn stddev(mut self) -> ExprNode<'a> {
        self.rollup = Rollup::StdDev;
        ExprNode::Aggregate(self)
    }

    pub fn min(mut self) -> ExprNode<'a> {
        self.rollup = Rollup::Min;
        ExprNode::Aggregate(self)
    }

    pub fn max(mut self) -> ExprNode<'a> {
        self.rollup = Rollup::Max;
        ExprNode::Aggregate(self)
    }

    pub fn sum(mut self) -> ExprNode<'a> {
        self.rollup = Rollup::Sum;
        ExprNode::Aggregate(self)
    }

    pub fn count(mut self) -> ExprNode<'a> {
        self.rollup = Rollup::Count;
        ExprNode::Aggregate(self)
    }

    pub fn n_unique(mut self) -> ExprNode<'a> {
        self.rollup = Rollup::NUnique;
        ExprNode::Aggregate(self)
    }

    pub fn lt(self, rhs: impl Into<ExprNode<'a>>) -> ExprNode<'a> {
        ExprNode::Compare(CompareExprNode::new(
            ExprNode::Aggregate(self),
            rhs.into(),
            CompareOp::Lt,
        ))
    }

    pub fn lte(self, rhs: impl Into<ExprNode<'a>>) -> ExprNode<'a> {
        ExprNode::Compare(CompareExprNode::new(
            ExprNode::Aggregate(self),
            rhs.into(),
            CompareOp::Lte,
        ))
    }

    pub fn gt(self, rhs: impl Into<ExprNode<'a>>) -> ExprNode<'a> {
        ExprNode::Compare(CompareExprNode::new(
            ExprNode::Aggregate(self),
            rhs.into(),
            CompareOp::Gt,
        ))
    }

    pub fn gte(self, rhs: impl Into<ExprNode<'a>>) -> ExprNode<'a> {
        ExprNode::Compare(CompareExprNode::new(
            ExprNode::Aggregate(self),
            rhs.into(),
            CompareOp::Gte,
        ))
    }

    pub fn eq(self, rhs: impl Into<ExprNode<'a>>) -> ExprNode<'a> {
        ExprNode::Compare(CompareExprNode::new(
            ExprNode::Aggregate(self),
            rhs.into(),
            CompareOp::Eq,
        ))
    }
}

impl ExprEval<MetricSet> for AggregateExprNode<'_> {
    fn eval<'a>(&'a mut self, metrics: &'a MetricSet) -> Value<'a> {
        let child_value = self.child.eval(metrics);

        let mut stat = Statistic::default();

        if let Value::Slice(values) = child_value {
            if self.rollup == Rollup::NUnique {
                let unique_count = values
                    .iter()
                    .fold(std::collections::HashSet::new(), |mut set, v| {
                        set.insert(v);

                        set
                    })
                    .len() as i32;

                return Value::Int32(unique_count);
            }

            for value in values.iter() {
                match value {
                    Value::Float32(v) => stat.add(*v),
                    Value::Int32(v) => stat.add(*v as f32),
                    Value::Int64(v) => stat.add(*v as f32),
                    _ => {}
                }
            }

            match self.rollup {
                Rollup::Mean => Value::Float32(stat.mean()),
                Rollup::StdDev => Value::Float32(stat.std_dev()),
                Rollup::Min => Value::Float32(stat.min()),
                Rollup::Max => Value::Float32(stat.max()),
                Rollup::Sum => Value::Float32(stat.sum()),
                Rollup::Count => Value::Float32(stat.count() as f32),
                _ => Value::Null, // NUnique is handled above
            }
        } else {
            Value::Null
        }
    }
}
