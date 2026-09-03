use super::{Evaluate, ExprResult};
use crate::ExprSelector;
use radiate_utils::{DataType, SmallStr};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub enum Selector {
    Matches(SmallStr),
    Index(usize),
    Field(SmallStr, DataType),
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

impl From<(SmallStr, DataType)> for Selector {
    fn from((s, dtype): (SmallStr, DataType)) -> Self {
        Selector::Field(s, dtype)
    }
}

impl From<usize> for Selector {
    fn from(idx: usize) -> Self {
        Selector::Index(idx)
    }
}

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

// use super::{Evaluate, ExprResult};
// use crate::ExprSelector;
// use radiate_utils::SmallStr;
// #[cfg(feature = "serde")]
// use serde::{Deserialize, Serialize};

// #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
// #[derive(Clone, Debug, PartialEq)]
// pub enum Selector {
//     Matches(SmallStr),
// }

// #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
// #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
// pub enum StatisticField {
//     LastValue,
//     Mean,
//     StdDev,
//     Min,
//     Max,
//     Sum,
//     Var,
//     Skew,
//     Count,
//     Version,
//     UpdateCount,
// }

// /// How the extracted statistic should be wrapped. `Value` returns it as an `f32`
// /// (or `u64` for count/generation/update_count); `Duration` reinterprets the f32 as seconds.
// #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
// #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
// pub enum StatisticKind {
//     Value,
//     Duration,
// }

// /// Selects one statistic from a named metric in a [`MetricSet`].
// #[derive(Clone, Debug, PartialEq)]
// #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
// pub struct SelectExpr {
//     pub metric: Option<SmallStr>,
//     pub field: StatisticField,
//     pub kind: StatisticKind,
// }

// impl SelectExpr {
//     pub fn new(metric: impl Into<SmallStr>) -> Self {
//         Self {
//             metric: Some(metric.into()),
//             field: StatisticField::LastValue,
//             kind: StatisticKind::Value,
//         }
//     }

//     pub fn with_field(mut self, field: StatisticField) -> Self {
//         self.field = field;
//         self
//     }

//     pub fn with_kind(mut self, kind: StatisticKind) -> Self {
//         self.kind = kind;
//         self
//     }
// }

// impl<'a, T> Evaluate<'a, T> for SelectExpr
// where
//     T: ExprSelector,
// {
//     fn eval(&'a mut self, metrics: &T) -> ExprResult<'a> {
//         Ok(metrics.select(self))
//     }
// }
