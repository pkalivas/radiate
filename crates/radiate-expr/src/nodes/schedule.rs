use crate::{Evaluate, ExprResult, ExprSelector};
use radiate_utils::AnyValue;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct EveryState {
    max: usize,
    count: usize,
}

impl EveryState {
    pub fn new(interval: usize) -> Self {
        Self {
            max: interval,
            count: 0,
        }
    }

    pub(crate) fn reset(&mut self) {
        self.count = 0;
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub enum ScheduleExpr {
    Every(EveryState),
}

impl<'a, T> Evaluate<'a, T> for ScheduleExpr
where
    T: ExprSelector,
{
    fn eval(&'a mut self, _metrics: &T) -> ExprResult<'a> {
        match self {
            ScheduleExpr::Every(state) => {
                state.count += 1;
                if state.count >= state.max {
                    state.count = 0;
                    Ok(AnyValue::Bool(true))
                } else {
                    Ok(AnyValue::Bool(false))
                }
            }
        }
    }
}
