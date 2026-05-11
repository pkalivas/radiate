use super::{Evaluate, ExprResult};
use crate::MetricSet;
use radiate_utils::{AnyValue, SmallStr};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Counts the number of consecutive evaluations during which a metric's
/// `last_value` has stayed within `epsilon` of the value last considered an
/// "improvement". Resets to zero on any change exceeding `epsilon`.
///
/// Returns `Float32(count as f32)` on every eval. Returns `Null` if the metric
/// is not present or its current value is non-finite — downstream Clamp /
/// Coalesce handles the fallback.
///
/// This is the building block for `expr::stagnation` and `expr::is_stagnant`.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct StagnationExpr {
    pub(super) metric: SmallStr,
    pub(super) epsilon: f32,
    pub(super) last_value: Option<f32>,
    pub(super) count: u32,
}

impl StagnationExpr {
    pub fn new(metric: impl Into<SmallStr>, epsilon: f32) -> Self {
        Self {
            metric: metric.into(),
            epsilon,
            last_value: None,
            count: 0,
        }
    }

    pub(super) fn reset(&mut self) {
        self.last_value = None;
        self.count = 0;
    }
}

impl Evaluate for StagnationExpr {
    fn eval<'a>(&'a mut self, metrics: &MetricSet) -> ExprResult<'a> {
        let Some(metric) = metrics.get(self.metric.as_str()) else {
            return Ok(AnyValue::Null);
        };
        let current = metric.last_value();
        if !current.is_finite() {
            return Ok(AnyValue::Null);
        }

        match self.last_value {
            None => {
                self.last_value = Some(current);
                self.count = 0;
            }
            Some(last) => {
                if (current - last).abs() > self.epsilon {
                    self.last_value = Some(current);
                    self.count = 0;
                } else {
                    self.count = self.count.saturating_add(1);
                }
            }
        }

        Ok(AnyValue::Float32(self.count as f32))
    }
}
