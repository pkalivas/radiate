use crate::{EvalExpr, ExprResult, ExprSelect};
use radiate_utils::AnyValue;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub enum ScheduleOp {
    Interval {
        count: usize,
        limit: usize,
    },
    Duration {
        #[cfg_attr(feature = "serde", serde(skip))]
        last: Option<std::time::Instant>,
        interval: std::time::Duration,
    },
}

impl ScheduleOp {
    pub(crate) fn try_schedule(&mut self) -> bool {
        match self {
            ScheduleOp::Interval { count, limit } => {
                *count += 1;
                if *count >= *limit {
                    *count = 0;
                    true
                } else {
                    false
                }
            }
            ScheduleOp::Duration { last, interval } => {
                let now = std::time::Instant::now();
                if let Some(l) = *last {
                    if now.duration_since(l) >= *interval {
                        *last = Some(now);
                        true
                    } else {
                        false
                    }
                } else {
                    *last = Some(now);
                    true
                }
            }
        }
    }

    pub(crate) fn reset(&mut self) {
        match self {
            ScheduleOp::Interval { count, limit: _ } => *count = 0,
            ScheduleOp::Duration { last, interval: _ } => *last = None,
        }
    }
}

impl From<usize> for ScheduleOp {
    fn from(interval: usize) -> Self {
        ScheduleOp::Interval {
            count: 0,
            limit: interval,
        }
    }
}

impl From<std::time::Duration> for ScheduleOp {
    fn from(duration: std::time::Duration) -> Self {
        ScheduleOp::Duration {
            last: None,
            interval: duration,
        }
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct ScheduleExpr {
    pub(crate) op: ScheduleOp,
}

impl ScheduleExpr {
    pub fn into_inner(self) -> ScheduleOp {
        self.op
    }

    pub(crate) fn reset(&mut self) {
        self.op.reset();
    }
}

impl<'a, T> EvalExpr<'a, T> for ScheduleExpr
where
    T: ExprSelect<'a>,
{
    fn evaluate(&'a mut self, _: &T) -> ExprResult<'a> {
        Ok(AnyValue::Bool(self.op.try_schedule()))
    }
}

impl<T: Into<ScheduleOp>> From<T> for ScheduleExpr {
    fn from(value: T) -> Self {
        ScheduleExpr { op: value.into() }
    }
}
