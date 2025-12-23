use crate::Valid;

#[derive(Clone, Debug, PartialEq)]
pub enum CycleShape {
    Triangle,
    Sine,
}

#[derive(Clone, Debug, PartialEq)]
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
    /// A warmup exponential schedule that starts at `start`, rises to `peak` over `warmup_steps`,
    /// then decays to `end` with a half-life of `half_life`.
    ///
    /// # Parameters
    /// - `warmup_steps`: Number of steps to reach peak from start.
    /// - `start`: The starting rate value.
    /// - `peak`: The peak rate value after warmup.
    /// - `end`: The ending rate value after decay.
    /// - `half_life`: The half-life period for decay after warmup.
    WarmupExp {
        warmup_steps: usize,
        start: f32,
        peak: f32,
        end: f32,
        half_life: usize,
    },
}

impl Rate {
    pub fn value(&self, step: usize) -> f32 {
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
            Rate::WarmupExp {
                warmup_steps,
                start,
                peak,
                end,
                half_life,
            } => {
                if step < *warmup_steps {
                    if *warmup_steps == 0 {
                        return *peak;
                    }
                    let t = f_step / *warmup_steps as f32;
                    start + (peak - start) * t
                } else {
                    let decay_step = step - *warmup_steps;
                    let decay = 0.5_f32.powf(decay_step as f32 / *half_life as f32);
                    end + (peak - end) * decay
                }
            }
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
            Rate::WarmupExp {
                start, peak, end, ..
            } => {
                (0.0..=1.0).contains(start)
                    && (0.0..=1.0).contains(peak)
                    && (0.0..=1.0).contains(end)
            }
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_values() {
        let fixed = Rate::Fixed(0.5);
        assert_eq!(fixed.value(0), 0.5);
        assert_eq!(fixed.value(10), 0.5);

        let linear = Rate::Linear(0.0, 1.0, 10);
        assert_eq!(linear.value(0), 0.0);
        assert_eq!(linear.value(5), 0.5);
        assert_eq!(linear.value(10), 1.0);
        assert_eq!(linear.value(15), 1.0);

        let exponential = Rate::Exponential(1.0, 0.1, 5);
        assert!((exponential.value(0) - 1.0).abs() < 1e-6);
        assert!((exponential.value(5) - 0.55).abs() < 1e-2);
        assert!((exponential.value(10) - 0.325).abs() < 1e-2);

        let cyclical = Rate::Cyclical(0.0, 1.0, 10, CycleShape::Triangle);
        assert!((cyclical.value(0) - 0.0).abs() < 1e-6);
        assert!((cyclical.value(2) - 0.4).abs() < 1e-6);
        assert!((cyclical.value(5) - 1.0).abs() < 1e-6);
        assert!((cyclical.value(7) - 0.6).abs() < 1e-6);
        assert!((cyclical.value(10) - 0.0).abs() < 1e-6);

        let cyclical_sine = Rate::Cyclical(0.0, 1.0, 10, CycleShape::Sine);
        assert!((cyclical_sine.value(0) - 0.0).abs() < 1e-6);
        assert!((cyclical_sine.value(2) - (std::f32::consts::TAU * 0.2).sin().abs()).abs() < 1e-6);
        assert!((cyclical_sine.value(5)).abs() < 1e-6);
        assert!((cyclical_sine.value(7) - (std::f32::consts::TAU * 0.7).sin().abs()).abs() < 1e-6);
        assert!((cyclical_sine.value(10) - 0.0).abs() < 1e-6);

        let stepwise = Rate::Stepwise(vec![(0, 0.0), (5, 0.5), (10, 1.0)]);
        assert_eq!(stepwise.value(0), 0.0);
        assert_eq!(stepwise.value(3), 0.0);
        assert_eq!(stepwise.value(5), 0.5);
        assert_eq!(stepwise.value(7), 0.5);
        assert_eq!(stepwise.value(10), 1.0);
        assert_eq!(stepwise.value(15), 1.0);

        let warmup_exp = Rate::WarmupExp {
            warmup_steps: 5,
            start: 0.0,
            peak: 1.0,
            end: 0.1,
            half_life: 5,
        };
        assert_eq!(warmup_exp.value(0), 0.0);
        assert_eq!(warmup_exp.value(2), 0.4);
        assert_eq!(warmup_exp.value(5), 1.0);
        assert!((warmup_exp.value(10) - 0.55).abs() < 1e-2);
        assert!((warmup_exp.value(15) - 0.325).abs() < 1e-2);
    }

    #[test]
    fn test_rates_between_0_and_1() {
        let fixed = Rate::Fixed(0.5);
        let linear = Rate::Linear(0.0, 1.0, 100);
        let exponential = Rate::Exponential(1.0, 0.0, 50);
        let cyclical = Rate::Cyclical(0.0, 1.0, 20, CycleShape::Triangle);
        let cyclical_sine = Rate::Cyclical(0.0, 1.0, 20, CycleShape::Sine);
        let stepwise = Rate::Stepwise(vec![(0, 0.0), (10, 0.5), (20, 1.0)]);
        let warmup_exp = Rate::WarmupExp {
            warmup_steps: 50,
            start: 0.0,
            peak: 1.0,
            end: 0.0,
            half_life: 50,
        };

        for i in 0..100_000 {
            let fixed_value = fixed.value(i);
            let linear_value = linear.value(i);
            let exp_value = exponential.value(i);
            let cycle_value = cyclical.value(i);
            let cycle_sine_value = cyclical_sine.value(i);
            let stepwise_value = stepwise.value(i);
            let warmup_exp_value = warmup_exp.value(i);

            assert!(fixed_value >= 0.0 && fixed_value <= 1.0);
            assert!(linear_value >= 0.0 && linear_value <= 1.0);
            assert!(exp_value >= 0.0 && exp_value <= 1.0);
            assert!(cycle_value >= 0.0 && cycle_value <= 1.0);
            assert!(cycle_sine_value >= 0.0 && cycle_sine_value <= 1.0);
            assert!(stepwise_value >= 0.0 && stepwise_value <= 1.0);
            assert!(warmup_exp_value >= 0.0 && warmup_exp_value <= 1.0);
        }
    }

    #[test]
    fn test_rate_clamping() {
        let linear = Rate::Linear(0.0, 1.0, 10);
        assert_eq!(linear.value(15), 1.0);
    }

    #[test]
    fn test_default_rate() {
        let default_rate = Rate::default();
        assert_eq!(default_rate.value(0), 1.0);
        assert_eq!(default_rate.value(100), 1.0);
    }

    #[test]
    fn test_rate_validity() {
        let valid_fixed = Rate::Fixed(0.5);
        let invalid_fixed = Rate::Fixed(1.5);
        assert!(valid_fixed.is_valid());
        assert!(!invalid_fixed.is_valid());
    }
}
