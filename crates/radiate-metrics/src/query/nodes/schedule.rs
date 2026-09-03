use crate::{Evaluate, ExprResult, ExprSelector};
use radiate_utils::AnyValue;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct IndexState {
    max: usize,
    count: usize,
}

impl IndexState {
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
pub struct DurationState {
    interval: std::time::Duration,
    #[cfg_attr(feature = "serde", serde(skip))]
    last: Option<std::time::Instant>,
}

impl From<std::time::Duration> for DurationState {
    fn from(duration: std::time::Duration) -> Self {
        Self {
            interval: duration,
            last: None,
        }
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub enum ScheduleExpr {
    Interval(IndexState),
    Duration(DurationState),
}

impl ScheduleExpr {
    pub fn interval(interval: usize) -> Self {
        ScheduleExpr::Interval(IndexState::new(interval))
    }

    pub fn duration(duration: std::time::Duration) -> Self {
        ScheduleExpr::Duration(DurationState {
            interval: duration,
            last: None,
        })
    }

    pub(crate) fn reset(&mut self) {
        match self {
            ScheduleExpr::Interval(state) => state.reset(),
            ScheduleExpr::Duration(state) => state.last = None,
        }
    }
}

impl<'a, T> Evaluate<'a, T> for ScheduleExpr
where
    T: ExprSelector,
{
    fn eval(&'a mut self, _: &T) -> ExprResult<'a> {
        match self {
            ScheduleExpr::Interval(state) => {
                state.count += 1;
                if state.count >= state.max {
                    state.count = 0;
                    Ok(AnyValue::Bool(true))
                } else {
                    Ok(AnyValue::Bool(false))
                }
            }
            ScheduleExpr::Duration(state) => {
                let now = std::time::Instant::now();
                if let Some(last) = state.last {
                    if now.duration_since(last) >= state.interval {
                        state.last = Some(now);
                        Ok(AnyValue::Bool(true))
                    } else {
                        Ok(AnyValue::Bool(false))
                    }
                } else {
                    state.last = Some(now);
                    Ok(AnyValue::Bool(true))
                }
            }
        }
    }
}
