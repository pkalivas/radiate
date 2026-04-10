use crate::{AnyValue, ExprProjection, ExprQuery};

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

#[derive(Clone, Debug, PartialEq)]
pub enum ScheduleExpr {
    Every(EveryState),
}

impl<T> ExprQuery<T> for ScheduleExpr
where
    T: ExprProjection,
{
    fn dispatch<'a>(&'a mut self, _input: &T) -> AnyValue<'a> {
        match self {
            ScheduleExpr::Every(state) => {
                state.count += 1;
                if state.count >= state.max {
                    state.count = 0;
                    AnyValue::Bool(true)
                } else {
                    AnyValue::Bool(false)
                }
            }
        }
    }
}
