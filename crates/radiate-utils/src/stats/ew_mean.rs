use crate::Float;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Exponentially weighted moving mean.
///
/// Recursive form: `μ_n = α · x_n + (1 − α) · μ_{n−1}`, seeded from the first
/// sample so there is no bias-correction warmup. Larger `α` weights recent
/// samples more heavily; `α = 1` collapses to "last value."
///
/// Useful as an adaptive-controller input where recent behavior should
/// dominate older history.
#[derive(Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct EwMean<F: Float = f32> {
    alpha: F,
    value: F,
    count: u32,
}

impl<F: Float> EwMean<F> {
    /// New estimator with smoothing factor `α ∈ (0, 1]`. Panics if out of range.
    pub fn new(alpha: F) -> Self {
        assert!(
            alpha > F::ZERO && alpha <= F::ONE,
            "alpha must be in (0, 1]"
        );
        Self {
            alpha,
            value: F::ZERO,
            count: 0,
        }
    }

    pub fn alpha(&self) -> F {
        self.alpha
    }

    pub fn count(&self) -> u32 {
        self.count
    }

    /// Current estimate. Returns `None` before any sample has been added.
    pub fn value(&self) -> Option<F> {
        if self.count == 0 {
            None
        } else {
            Some(self.value)
        }
    }

    pub fn add(&mut self, x: F) {
        if self.count == 0 {
            self.value = x;
        } else {
            self.value = self.alpha * x + (F::ONE - self.alpha) * self.value;
        }
        self.count += 1;
    }

    pub fn clear(&mut self) {
        self.value = F::ZERO;
        self.count = 0;
    }
}

impl<F: Float> Extend<F> for EwMean<F> {
    fn extend<T: IntoIterator<Item = F>>(&mut self, iter: T) {
        for v in iter {
            self.add(v);
        }
    }
}

impl<F: Float> std::fmt::Debug for EwMean<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EwMean")
            .field("alpha", &self.alpha)
            .field("count", &self.count)
            .field("value", &self.value())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_returns_none() {
        let m = EwMean::<f32>::new(0.5);
        assert!(m.value().is_none());
    }

    #[test]
    fn single_sample_returns_that_sample() {
        let mut m = EwMean::<f32>::new(0.5);
        m.add(7.0);
        assert_eq!(m.value().unwrap(), 7.0);
    }

    #[test]
    fn alpha_one_returns_last_value() {
        let mut m = EwMean::<f32>::new(1.0);
        m.add(1.0);
        m.add(2.0);
        m.add(99.0);
        assert_eq!(m.value().unwrap(), 99.0);
    }

    #[test]
    fn constant_input_converges_to_constant() {
        let mut m = EwMean::<f32>::new(0.1);
        for _ in 0..200 {
            m.add(5.0);
        }
        assert!((m.value().unwrap() - 5.0).abs() < 1e-6);
    }

    #[test]
    fn responds_to_step_change() {
        let mut m = EwMean::<f32>::new(0.3);
        for _ in 0..50 {
            m.add(0.0);
        }
        // step to 10.0
        m.add(10.0);
        let after_step = m.value().unwrap();
        // first sample after step should pull toward 10 by alpha
        assert!((after_step - 3.0).abs() < 1e-4, "got {after_step}");
    }

    #[test]
    fn clear_resets() {
        let mut m = EwMean::<f32>::new(0.5);
        m.add(1.0);
        m.add(2.0);
        m.clear();
        assert_eq!(m.count(), 0);
        assert!(m.value().is_none());
    }

    #[test]
    #[should_panic]
    fn rejects_alpha_zero() {
        let _ = EwMean::<f32>::new(0.0);
    }

    #[test]
    #[should_panic]
    fn rejects_alpha_above_one() {
        let _ = EwMean::<f32>::new(1.5);
    }
}
