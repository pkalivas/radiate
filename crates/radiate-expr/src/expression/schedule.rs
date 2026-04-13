use crate::{AnyValue, ExprProjection, ExprQuery, ExprResult};
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
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub enum ScheduleExpr {
    Every(EveryState),
}

impl<T> ExprQuery<T> for ScheduleExpr
where
    T: ExprProjection,
{
    fn dispatch<'a>(&'a mut self, _input: &T) -> ExprResult<'a> {
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
