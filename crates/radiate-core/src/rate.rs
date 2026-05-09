use crate::freeze::Frozen;
use crate::stats::expression::{Evaluate, Expr};
use crate::{Freezable, MetricSet, Valid};
use radiate_utils::AnyValue;
use std::fmt::Debug;

#[derive(Clone, Debug, PartialEq)]
pub enum CycleShape {
    Triangle,
    Sine,
}

/// Rate enum representing different types of rate schedules where each variant defines a
/// method to compute the rate value at a given step.
/// These are designed to produce values within the range [0.0, 1.0] - ie: a rate.
#[derive(Clone)]
pub enum Rate {
    /// A fixed rate that does not change over time.
    ///
    /// # Parameters
    /// - `f32`: The fixed rate value.
    Fixed(f32),
    /// A linear rate that changes from start to end over a number of steps.
    ///
    /// # Parameters
    /// - `start`: The starting rate value.
    /// - `end`: The ending rate value.
    /// - `steps`: The number of steps over which to change the rate.
    Linear(f32, f32, usize),
    /// An exponential rate that changes from start to end over a half-life period.
    ///
    /// # Parameters
    /// - `start`: The starting rate value.
    /// - `end`: The ending rate value.
    /// - `half_life`: The half-life period over which to change the rate.
    Exponential(f32, f32, usize),
    /// A cyclical rate that oscillates between min and max over a period.
    ///
    /// # Parameters
    /// - `min`: The minimum rate value.
    /// - `max`: The maximum rate value.
    /// - `period`: The period over which to cycle the rate.
    /// - `shape`: The shape of the cycle (Triangle or Sine).
    Cyclical(f32, f32, usize, CycleShape),
    /// Piecewise-constant schedule: at each listed step, rate jumps to the given value.
    /// The value remains constant until the next listed step.
    /// The first step must be 0.
    /// If the current step is beyond the last listed step, the rate remains at the last value.
    ///
    /// # Parameters
    /// - `Vec<(usize, f32)>`: A vector of (step, rate) pairs.
    Stepwise(Vec<(usize, f32)>),

    /// A rate defined by an expression that can query metrics.
    /// The expression should evaluate to a float value representing the rate.
    /// The expression can use the provided metrics to compute a dynamic rate based on the current state of the ecosystem.
    /// The expression is expected to return a value in the range [0.0, 1.0], but this is not enforced at compile time.
    Expr(Expr),
}

impl Rate {
    pub fn get(&mut self, generation: usize, metrics: &MetricSet) -> f32 {
        match self {
            Rate::Expr(expr) => expr
                .eval(metrics)
                .ok()
                .and_then(|v| v.extract())
                .unwrap_or(0.0),
            _ => self.get_by_index(generation),
        }
    }

    pub fn get_by_index(&self, step: usize) -> f32 {
        let f_step = step as f32;
        match self {
            Rate::Fixed(v) => *v,
            Rate::Linear(start, end, steps) => {
                if *steps == 0 {
                    return *end;
                }

                let t = (f_step / *steps as f32).min(1.0);
                start + (end - start) * t
            }
            Rate::Exponential(start, end, half_life) => {
                if *half_life == 0 {
                    return *end;
                }

                let decay = 0.5_f32.powf(f_step / *half_life as f32);
                end + (start - end) * decay
            }
            Rate::Cyclical(min, max, period, shape) => {
                let phase = (f_step % *period as f32) / *period as f32;
                let tri = if phase < 0.5 {
                    phase * 2.0
                } else {
                    (1.0 - phase) * 2.0
                };

                let s = match shape {
                    CycleShape::Triangle => tri,
                    CycleShape::Sine => (std::f32::consts::TAU * phase).sin().abs(),
                };

                min + (max - min) * s
            }
            Rate::Stepwise(steps) => {
                if steps.is_empty() {
                    return 0.0;
                }

                let mut last_value = steps[0].1;
                for (s, v) in steps {
                    if step < *s {
                        break;
                    }

                    last_value = *v;
                }

                last_value
            }
            _ => 1.0,
        }
    }

    /// Render the schedule as a structured frozen entry — variant name as the
    /// `"type"` tag, parameters as named fields. Falls back to the schedule's
    /// debug repr for the `Expr` variant.
    pub fn freeze(&self) -> Frozen {
        match self {
            Rate::Fixed(v) => Frozen::new().with("type", "Fixed").with("value", *v),
            Rate::Linear(start, end, steps) => Frozen::new()
                .with("type", "Linear")
                .with("start", *start)
                .with("end", *end)
                .with("steps", *steps),
            Rate::Exponential(start, end, half_life) => Frozen::new()
                .with("type", "Exponential")
                .with("start", *start)
                .with("end", *end)
                .with("half_life", *half_life),
            Rate::Cyclical(min, max, period, shape) => Frozen::new()
                .with("type", "Cyclical")
                .with("min", *min)
                .with("max", *max)
                .with("period", *period)
                .with(
                    "shape",
                    match shape {
                        CycleShape::Triangle => "Triangle",
                        CycleShape::Sine => "Sine",
                    },
                ),
            Rate::Stepwise(steps) => {
                let entries: Vec<AnyValue<'static>> = steps
                    .iter()
                    .map(|(step, rate)| {
                        Frozen::new()
                            .with("step", *step)
                            .with("rate", *rate)
                            .build()
                    })
                    .collect();
                Frozen::new()
                    .with("type", "Stepwise")
                    .with("steps", AnyValue::Vector(entries))
            }
            Rate::Expr(expr) => Frozen::new()
                .with("type", "Expr")
                .with("expr", format!("{:?}", expr)),
        }
    }
}

impl Valid for Rate {
    fn is_valid(&self) -> bool {
        match self {
            Rate::Fixed(v) => (0.0..=1.0).contains(v),
            Rate::Linear(start, end, _) => (0.0..=1.0).contains(start) && (0.0..=1.0).contains(end),
            Rate::Exponential(start, end, _) => {
                (0.0..=1.0).contains(start) && (0.0..=1.0).contains(end)
            }
            Rate::Cyclical(min, max, _, _) => {
                (0.0..=1.0).contains(min) && (0.0..=1.0).contains(max) && min <= max
            }
            Rate::Stepwise(steps) => {
                if steps.is_empty() {
                    return false;
                }

                if steps[0].0 != 0 {
                    return false;
                }

                let mut last_step = 0;
                for (s, v) in steps {
                    if *s < last_step || !(0.0..=1.0).contains(v) {
                        return false;
                    }
                    last_step = *s;
                }

                true
            }
            _ => true,
        }
    }
}

impl Freezable for Rate {
    fn as_frozen(&self) -> Frozen {
        self.freeze()
    }
}

impl Default for Rate {
    fn default() -> Self {
        Rate::Fixed(1.0)
    }
}

impl From<f32> for Rate {
    fn from(value: f32) -> Self {
        Rate::Fixed(value)
    }
}

impl From<Vec<(usize, f32)>> for Rate {
    fn from(steps: Vec<(usize, f32)>) -> Self {
        Rate::Stepwise(steps)
    }
}

impl From<Expr> for Rate {
    fn from(expr: Expr) -> Self {
        Rate::Expr(expr)
    }
}

impl Debug for Rate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Rate::Fixed(v) => write!(f, "Rate::Fixed({})", v),
            Rate::Linear(start, end, steps) => {
                write!(
                    f,
                    "Rate::Linear(start: {}, end: {}, steps: {})",
                    start, end, steps
                )
            }
            Rate::Exponential(start, end, half_life) => write!(
                f,
                "Rate::Exponential(start: {}, end: {}, half_life: {})",
                start, end, half_life
            ),
            Rate::Cyclical(min, max, period, shape) => write!(
                f,
                "Rate::Cyclical(min: {}, max: {}, period: {}, shape: {:?})",
                min, max, period, shape
            ),
            Rate::Stepwise(steps) => write!(f, "Rate::Stepwise(steps: {:?})", steps),
            Rate::Expr(_) => write!(f, "Rate::Expr(<function>)"),
        }
    }
}

impl PartialEq for Rate {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Rate::Fixed(a), Rate::Fixed(b)) => a == b,
            (Rate::Linear(a_start, a_end, a_steps), Rate::Linear(b_start, b_end, b_steps)) => {
                a_start == b_start && a_end == b_end && a_steps == b_steps
            }
            (
                Rate::Exponential(a_start, a_end, a_half_life),
                Rate::Exponential(b_start, b_end, b_half_life),
            ) => a_start == b_start && a_end == b_end && a_half_life == b_half_life,
            (
                Rate::Cyclical(a_min, a_max, a_period, a_shape),
                Rate::Cyclical(b_min, b_max, b_period, b_shape),
            ) => a_min == b_min && a_max == b_max && a_period == b_period && a_shape == b_shape,
            (Rate::Stepwise(a_steps), Rate::Stepwise(b_steps)) => a_steps == b_steps,
            // For Expr variants, we consider them equal if they are the same variant,
            // since we cannot compare the inner function for equality.
            (Rate::Expr(_), Rate::Expr(_)) => true,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_values() {
        let fixed = Rate::Fixed(0.5);
        assert_eq!(fixed.get_by_index(0), 0.5);
        assert_eq!(fixed.get_by_index(10), 0.5);

        let linear = Rate::Linear(0.0, 1.0, 10);
        assert_eq!(linear.get_by_index(0), 0.0);
        assert_eq!(linear.get_by_index(5), 0.5);
        assert_eq!(linear.get_by_index(10), 1.0);
        assert_eq!(linear.get_by_index(15), 1.0);

        let exponential = Rate::Exponential(1.0, 0.1, 5);
        assert!((exponential.get_by_index(0) - 1.0).abs() < 1e-6);
        assert!((exponential.get_by_index(5) - 0.55).abs() < 1e-2);
        assert!((exponential.get_by_index(10) - 0.325).abs() < 1e-2);

        let cyclical = Rate::Cyclical(0.0, 1.0, 10, CycleShape::Triangle);
        assert!((cyclical.get_by_index(0) - 0.0).abs() < 1e-6);
        assert!((cyclical.get_by_index(2) - 0.4).abs() < 1e-6);
        assert!((cyclical.get_by_index(5) - 1.0).abs() < 1e-6);
        assert!((cyclical.get_by_index(7) - 0.6).abs() < 1e-6);
        assert!((cyclical.get_by_index(10) - 0.0).abs() < 1e-6);

        let cyclical_sine = Rate::Cyclical(0.0, 1.0, 10, CycleShape::Sine);
        assert!((cyclical_sine.get_by_index(0) - 0.0).abs() < 1e-6);
        assert!(
            (cyclical_sine.get_by_index(2) - (std::f32::consts::TAU * 0.2).sin().abs()).abs()
                < 1e-6
        );
        assert!(cyclical_sine.get_by_index(5).abs() < 1e-6);
        assert!(
            (cyclical_sine.get_by_index(7) - (std::f32::consts::TAU * 0.7).sin().abs()).abs()
                < 1e-6
        );
        assert!((cyclical_sine.get_by_index(10) - 0.0).abs() < 1e-6);

        let stepwise = Rate::Stepwise(vec![(0, 0.0), (5, 0.5), (10, 1.0)]);
        assert_eq!(stepwise.get_by_index(0), 0.0);
        assert_eq!(stepwise.get_by_index(3), 0.0);
        assert_eq!(stepwise.get_by_index(5), 0.5);
        assert_eq!(stepwise.get_by_index(7), 0.5);
        assert_eq!(stepwise.get_by_index(10), 1.0);
        assert_eq!(stepwise.get_by_index(15), 1.0);
    }

    #[test]
    fn test_rates_between_0_and_1() {
        let fixed = Rate::Fixed(0.5);
        let linear = Rate::Linear(0.0, 1.0, 100);
        let exponential = Rate::Exponential(1.0, 0.0, 50);
        let cyclical = Rate::Cyclical(0.0, 1.0, 20, CycleShape::Triangle);
        let cyclical_sine = Rate::Cyclical(0.0, 1.0, 20, CycleShape::Sine);
        let stepwise = Rate::Stepwise(vec![(0, 0.0), (10, 0.5), (20, 1.0)]);

        for i in 0..100_000 {
            let fixed_value = fixed.get_by_index(i);
            let linear_value = linear.get_by_index(i);
            let exp_value = exponential.get_by_index(i);
            let cycle_value = cyclical.get_by_index(i);
            let cycle_sine_value = cyclical_sine.get_by_index(i);
            let stepwise_value = stepwise.get_by_index(i);

            assert!(fixed_value >= 0.0 && fixed_value <= 1.0);
            assert!(linear_value >= 0.0 && linear_value <= 1.0);
            assert!(exp_value >= 0.0 && exp_value <= 1.0);
            assert!(cycle_value >= 0.0 && cycle_value <= 1.0);
            assert!(cycle_sine_value >= 0.0 && cycle_sine_value <= 1.0);
            assert!(stepwise_value >= 0.0 && stepwise_value <= 1.0);
        }
    }

    #[test]
    fn test_rate_clamping() {
        let linear = Rate::Linear(0.0, 1.0, 10);
        assert_eq!(linear.get_by_index(15), 1.0);
    }

    #[test]
    fn test_default_rate() {
        let default_rate = Rate::default();
        assert_eq!(default_rate.get_by_index(0), 1.0);
        assert_eq!(default_rate.get_by_index(100), 1.0);
    }

    #[test]
    fn test_rate_validity() {
        let valid_fixed = Rate::Fixed(0.5);
        let invalid_fixed = Rate::Fixed(1.5);
        assert!(valid_fixed.is_valid());
        assert!(!invalid_fixed.is_valid());
    }
}
