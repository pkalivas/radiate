use super::{EvalExpr, ExprResult};
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

    pub fn nest_or_swap_child(self, child: impl Into<Selector>) -> Self {
        if let Selector::Nested { parent, .. } = &self.selector {
            return Self {
                selector: Selector::Nested {
                    parent: parent.clone(),
                    child: Box::new(child.into()),
                },
            };
        }

        Self {
            selector: Selector::Nested {
                parent: Box::new(self.selector),
                child: Box::new(child.into()),
            },
        }
    }
}

impl<'a, T: ExprSelect<'a>> EvalExpr<'a, T> for SelectExpr {
    fn evaluate(&'a mut self, metrics: &'a T) -> ExprResult<'a> {
        let selected = metrics.select(&self.selector);
        Ok(selected)
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub enum Selector {
    Identity,
    Index(usize),
    Range(usize, usize),
    Field(SmallStr),
    Nested {
        parent: Box<Selector>,
        child: Box<Selector>,
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

// impl From<(SmallStr, SmallStr, DataType)> for Selector {
//     fn from((name, field, dtype): (SmallStr, SmallStr, DataType)) -> Self {
//         Selector::Metric { name, field, dtype }
//     }
// }
