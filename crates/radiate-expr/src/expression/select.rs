use crate::{AnyValue, ExprProjection, ExprQuery};
use radiate_utils::SmallStr;

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
    fn dispatch<'a>(&'a mut self, input: &T) -> AnyValue<'a> {
        input.project(self).unwrap_or(AnyValue::Null)
    }
}
