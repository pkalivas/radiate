use std::{
    cell::Cell,
    fmt::{self, Debug},
};

#[derive(Clone, PartialEq)]
pub enum Rate {
    /// Static rate does not change over time.
    /// It is useful when we want to maintain a constant mutation or crossover rate.
    /// This can be beneficial in scenarios where we want to maintain
    /// a consistent level of exploration or exploitation.
    Static(Cell<f32>),
    /// Exponential smoothing updates the rate based on a weighted average
    /// of the current candidate and the previous rate.
    /// The alpha parameter controls the weight given to the current candidate.
    /// A higher alpha value means that the current candidate has a stronger influence,
    /// which can be useful in scenarios where we want to adapt quickly to changes.
    /// This can be particularly beneficial in dynamic environments where
    /// the optimal rate may vary significantly over time.
    /// By using exponential smoothing, we can achieve a more responsive
    /// and adaptive rate adjustment mechanism. It allows the algorithm to
    /// adjust the rate more effectively based on the current candidate's performance.
    ExpSmoothing { value: Cell<f32>, alpha: f32 },
    /// Momentum updates the value based on the candidate and previous momentum.
    /// The momentum is a form of inertia that helps smooth out the updates.
    /// This is useful in scenarios where we want to avoid abrupt changes
    /// in the rate, allowing for a more stable and gradual adjustment.
    /// The beta parameter controls the influence of the previous momentum.
    /// A higher beta value means that the previous momentum has a stronger influence,
    /// which can be beneficial in scenarios where we want to maintain a consistent
    /// rate of change. This can be particularly useful in optimization problems
    /// where we want to avoid overshooting the target or making erratic adjustments.
    /// By using momentum, we can achieve a smoother and more stable adjustment
    /// to the rate, which can lead to better convergence properties in optimization
    /// algorithms. It helps to balance exploration and exploitation by allowing
    /// the algorithm to make more informed decisions based on both the current
    /// candidate and the historical performance.
    Momentum {
        value: Cell<f32>,
        momentum: Cell<f32>,
        beta: f32,
    },
    /// Alpha: A smoothing factor that controls the sensitivity of the rate adjustment.
    /// Target Diversity: The desired level of diversity in the population.
    /// Max/Min Rate: Bounds for the mutation rate to prevent extreme values.
    /// Diversity Factor: Encourages rate adjustment based on how far the current diversity is from the target.
    /// Improvement Factor: Encourages rate adjustment based on the lack of fitness improvement.
    /// This approach provides a more dynamic and responsive rate update mechanism,
    /// potentially leading to better performance in maintaining diversity and improving fitness over generations.
    Diversity {
        current: Cell<f32>,
        alpha: f32,
        min: f32,
        max: f32,
    },
}

impl Rate {
    pub fn get(&self) -> f32 {
        match self {
            Rate::Static(cell) => cell.get(),
            Rate::ExpSmoothing { value, .. } => value.get(),
            Rate::Momentum {
                value,
                momentum,
                beta,
            } => {
                let current_value = value.get();
                let current_momentum = momentum.get();
                current_value + current_momentum * beta
            }
            Rate::Diversity { current, .. } => current.get(),
        }
    }

    /// Updates the rate using a candidate measurement.
    ///
    /// For the ExpSmoothing variant, the new rate is:
    ///   new_val = alpha * candidate + (1 - alpha) * current
    ///
    /// For Momentum, we update the momentum and then the value:
    ///   new_mom = beta * prev_mom + (1 - beta) * (candidate - current)
    ///   new_val = current + new_mom
    ///
    /// The Cyclical variant does not update based on candidate.
    pub fn update(&self, candidate: f32) {
        match self {
            Rate::Static(_) => {}
            Rate::ExpSmoothing { value, alpha } => {
                let current = value.get();
                let new_val = *alpha * candidate + (1.0 - *alpha) * current;
                value.set(new_val);
            }
            Rate::Momentum {
                value,
                momentum,
                beta,
            } => {
                let current = value.get();
                let delta = candidate - current;
                let prev_mom = momentum.get();
                let new_mom = *beta * prev_mom + (1.0 - *beta) * delta;
                let new_val = current + new_mom;
                value.set(new_val);
                momentum.set(new_mom);
            }
            Rate::Diversity {
                current,
                alpha,
                min,
                max,
            } => {
                let current_value = current.get();
                let adjustment = alpha * candidate.abs();
                let new_val = (current_value + adjustment).clamp(*min, *max);
                current.set(new_val);
            }
        }
    }
}

impl Debug for Rate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Rate::Static(cell) => write!(f, "Static({:.4})", cell.get()),
            Rate::ExpSmoothing { value, alpha } => {
                write!(f, "ExpSmoothing({:.4}, alpha={:.2})", value.get(), alpha)
            }
            Rate::Momentum { value, beta, .. } => {
                write!(f, "Momentum({:.4}, beta={:.2})", value.get(), beta)
            }
            Rate::Diversity {
                current,
                alpha,
                min,
                max,
            } => {
                write!(
                    f,
                    "Diversity(current={:.4}, alpha={:.2}, min={:.4}, max={:.4})",
                    current.get(),
                    alpha,
                    min,
                    max
                )
            }
        }
    }
}

impl From<f32> for Rate {
    fn from(value: f32) -> Self {
        Rate::Static(Cell::new(value))
    }
}
