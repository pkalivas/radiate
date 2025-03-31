use std::{
    cell::Cell,
    fmt::{self, Debug},
};

#[derive(Clone, PartialEq)]
pub enum Rate {
    Static(Cell<f32>),
    ExpSmoothing {
        value: Cell<f32>,
        alpha: f32,
    },
    Momentum {
        value: Cell<f32>,
        momentum: Cell<f32>,
        beta: f32,
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
    pub fn update(&mut self, candidate: f32) {
        match self {
            Rate::Static(_) => {
                // Do nothing; static rate remains unchanged.
                println!("Static rate remains unchanged.");
            }
            Rate::ExpSmoothing { value, alpha } => {
                let current = value.get();
                let new_val = *alpha * candidate + (1.0 - *alpha) * current;
                value.set(new_val);
                println!(
                    "ExpSmoothing updated: candidate = {:.4}, new value = {:.4}",
                    candidate, new_val
                );
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
                println!(
                    "Momentum updated: candidate = {:.4}, new value = {:.4}",
                    candidate, new_val
                );
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
        }
    }
}

impl From<f32> for Rate {
    fn from(value: f32) -> Self {
        Rate::Static(Cell::new(value))
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use super::*;

    #[test]
    fn test_rate_static() {
        let rate = Rate::from(0.5);
        assert_eq!(rate.get(), 0.5);
    }

    #[test]
    fn test_rate_exp_smoothing() {
        let mut exp_rate = Rate::ExpSmoothing {
            value: Cell::new(1.0),
            alpha: 0.2,
        };

        println!("Initial ExpSmoothing rate: {:?}", exp_rate);
        // Simulate candidate rate measurements.
        let candidates = vec![0.8, 0.6, 1.2, 0.9, 1.0];
        for candidate in candidates {
            exp_rate.update(candidate);
            println!("Updated ExpSmoothing rate: {:.4}", exp_rate.get());
        }

        // Create a momentum-based rate with an initial value of 1.0, beta = 0.3.
        let mut momentum_rate = Rate::Momentum {
            value: Cell::new(1.0),
            momentum: Cell::new(0.0),
            beta: 0.3,
        };

        println!("\nInitial Momentum rate: {:?}", momentum_rate);
        for candidate in vec![0.8, 0.6, 1.2, 0.9, 1.0] {
            momentum_rate.update(candidate);
            println!("Updated Momentum rate: {:.4}", momentum_rate.get());
        }
    }
}
