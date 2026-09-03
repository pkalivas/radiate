use super::{Evaluate, ExprResult};
use crate::ExprSelector;
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

impl<'a, T> Evaluate<'a, T> for SelectExpr
where
    T: ExprSelector,
{
    fn eval(&'a mut self, metrics: &T) -> ExprResult<'a> {
        Ok(metrics.select(&self.selector))
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub enum Selector {
    Identity,
    Matches(SmallStr),
    Index(usize),
    Metric {
        name: SmallStr,
        field: SmallStr,
        dtype: DataType,
    },
}

impl From<SmallStr> for Selector {
    fn from(s: SmallStr) -> Self {
        Selector::Matches(s)
    }
}

impl From<&str> for Selector {
    fn from(s: &str) -> Self {
        Selector::Matches(SmallStr::from_string(s.to_string()))
    }
}

impl From<usize> for Selector {
    fn from(idx: usize) -> Self {
        Selector::Index(idx)
    }
}

impl From<(SmallStr, SmallStr, DataType)> for Selector {
    fn from((name, field, dtype): (SmallStr, SmallStr, DataType)) -> Self {
        Selector::Metric { name, field, dtype }
    }
}
