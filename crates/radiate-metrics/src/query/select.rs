use super::{ExprEval, ExprResult};
use crate::ExprSelect;
use radiate_utils::{DataType, SmallStr};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SelectExpr {
    pub(crate) selector: Selector,
}

impl SelectExpr {
    pub fn new(metric: impl Into<Selector>) -> Self {
        Self {
            selector: metric.into(),
        }
    }

    pub fn selector(&self) -> &Selector {
        &self.selector
    }
}

impl<'a, T> ExprEval<'a, T> for SelectExpr
where
    T: ExprSelect,
{
    fn eval(&'a mut self, metrics: &T) -> ExprResult<'a> {
        Ok(metrics.select(&self.selector))
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub enum Selector {
    Identity,
    Index(usize),
    Range(usize, usize),
    Metric {
        name: SmallStr,
        field: SmallStr,
        dtype: DataType,
    },
}

impl From<usize> for Selector {
    fn from(idx: usize) -> Self {
        Selector::Index(idx)
    }
}

impl From<std::ops::Range<usize>> for Selector {
    fn from(range: std::ops::Range<usize>) -> Self {
        Selector::Range(range.start, range.end)
    }
}

impl From<(SmallStr, SmallStr, DataType)> for Selector {
    fn from((name, field, dtype): (SmallStr, SmallStr, DataType)) -> Self {
        Selector::Metric { name, field, dtype }
    }
}
