use super::{Evaluate, ExprResult};
use crate::ExprSelector;
use radiate_utils::SmallStr;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Which statistic to extract from a metric in a [`MetricSet`].
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MetricField {
    LastValue,
    Mean,
    StdDev,
    Min,
    Max,
    Sum,
    Var,
    Skew,
    Count,
    Generation,
    UpdateCount,
}

/// How the extracted statistic should be wrapped. `Value` returns it as an `f32`
/// (or `u64` for count/generation/update_count); `Duration` reinterprets the f32 as seconds.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum MetricKind {
    Value,
    Duration,
}

/// Selects one statistic from a named metric in a [`MetricSet`].
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SelectExpr {
    pub metric: Option<SmallStr>,
    pub field: MetricField,
    pub kind: MetricKind,
}

impl SelectExpr {
    pub fn new(metric: impl Into<SmallStr>) -> Self {
        Self {
            metric: Some(metric.into()),
            field: MetricField::LastValue,
            kind: MetricKind::Value,
        }
    }

    pub fn with_field(mut self, field: MetricField) -> Self {
        self.field = field;
        self
    }

    pub fn with_kind(mut self, kind: MetricKind) -> Self {
        self.kind = kind;
        self
    }
}

impl<'a, T> Evaluate<'a, T> for SelectExpr
where
    T: ExprSelector,
{
    fn eval(&'a mut self, metrics: &T) -> ExprResult<'a> {
        Ok(metrics.select(self))
    }
}
