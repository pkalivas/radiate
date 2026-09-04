use crate::{EvalExpr, ExprResult, ExprSelect};
use radiate_utils::SmallStr;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SelectExpr {
    pub(crate) selector: SelectOp,
}

impl SelectExpr {
    pub fn new(metric: impl Into<SelectOp>) -> Self {
        Self {
            selector: metric.into(),
        }
    }

    pub fn nest_or_swap_child(self, child: impl Into<SelectOp>) -> Self {
        if let SelectOp::Nested { parent, .. } = &self.selector {
            return Self {
                selector: SelectOp::Nested {
                    parent: parent.clone(),
                    child: Box::new(child.into()),
                },
            };
        }

        Self {
            selector: SelectOp::Nested {
                parent: Box::new(self.selector),
                child: Box::new(child.into()),
            },
        }
    }
}

impl<'a, T: ExprSelect<'a>> EvalExpr<'a, T> for SelectExpr {
    fn evaluate(&'a mut self, metrics: &'a T) -> ExprResult<'a> {
        metrics.select(&self.selector)
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub enum SelectOp {
    Identity,
    Index(usize),
    Range(usize, usize),
    Field(SmallStr),
    Nested {
        parent: Box<SelectOp>,
        child: Box<SelectOp>,
    },
}

impl From<usize> for SelectOp {
    fn from(idx: usize) -> Self {
        SelectOp::Index(idx)
    }
}

impl From<std::ops::Range<usize>> for SelectOp {
    fn from(range: std::ops::Range<usize>) -> Self {
        SelectOp::Range(range.start, range.end)
    }
}

impl From<SmallStr> for SelectOp {
    fn from(field: SmallStr) -> Self {
        SelectOp::Field(field)
    }
}

impl From<(SmallStr, SmallStr)> for SelectOp {
    fn from((parent, child): (SmallStr, SmallStr)) -> Self {
        SelectOp::Nested {
            parent: Box::new(SelectOp::Field(parent)),
            child: Box::new(SelectOp::Field(child)),
        }
    }
}
