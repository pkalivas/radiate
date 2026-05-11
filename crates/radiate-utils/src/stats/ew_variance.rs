use crate::Float;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Exponentially weighted moving mean and variance.
///
/// Roberts (1959) recursive form:
/// ```text
/// diff   = x − μ
/// incr   = α · diff
/// μ      ← μ + incr
/// σ²     ← (1 − α) · (σ² + diff · incr)
/// ```
/// Seeded from the first sample (variance starts at 0). Larger `α` weights
/// recent samples more heavily.
///
/// Useful as a controller input when you want both the recent level and the
/// recent dispersion (e.g. "score is volatile and trending up").
#[derive(Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct EwVariance<F: Float = f32> {
    alpha: F,
    mean: F,
    var: F,
    count: u32,
}

impl<F: Float> EwVariance<F> {
    /// New estimator with smoothing factor `α ∈ (0, 1]`. Panics if out of range.
    pub fn new(alpha: F) -> Self {
        assert!(
            alpha > F::ZERO && alpha <= F::ONE,
            "alpha must be in (0, 1]"
        );
        Self {
            alpha,
            mean: F::ZERO,
            var: F::ZERO,
            count: 0,
        }
    }

    pub fn alpha(&self) -> F {
        self.alpha
    }

    pub fn count(&self) -> u32 {
        self.count
    }

    /// Current mean estimate. Returns `None` before any sample has been added.
    pub fn mean(&self) -> Option<F> {
        if self.count == 0 {
            None
        } else {
            Some(self.mean)
        }
    }

    /// Current variance estimate. Returns `None` before two samples have been added.
    pub fn variance(&self) -> Option<F> {
        if self.count < 2 {
            None
        } else {
            Some(self.var)
        }
    }

    /// Current standard deviation estimate. Returns `None` before two samples.
    pub fn std_dev(&self) -> Option<F> {
        Some(self.variance()?.sqrt())
    }

    pub fn add(&mut self, x: F) {
        if self.count == 0 {
            self.mean = x;
        } else {
            let diff = x - self.mean;
            let incr = self.alpha * diff;
            self.mean = self.mean + incr;
            self.var = (F::ONE - self.alpha) * (self.var + diff * incr);
        }
        self.count += 1;
    }

    pub fn clear(&mut self) {
        self.mean = F::ZERO;
        self.var = F::ZERO;
        self.count = 0;
    }
}

impl<F: Float> Extend<F> for EwVariance<F> {
    fn extend<T: IntoIterator<Item = F>>(&mut self, iter: T) {
        for v in iter {
            self.add(v);
        }
    }
}

impl<F: Float> std::fmt::Debug for EwVariance<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EwVariance")
            .field("alpha", &self.alpha)
            .field("count", &self.count)
            .field("mean", &self.mean())
            .field("variance", &self.variance())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_returns_none() {
        let v = EwVariance::<f32>::new(0.5);
        assert!(v.mean().is_none());
        assert!(v.variance().is_none());
        assert!(v.std_dev().is_none());
    }

    #[test]
    fn single_sample_has_mean_no_variance() {
        let mut v = EwVariance::<f32>::new(0.5);
        v.add(4.0);
        assert_eq!(v.mean().unwrap(), 4.0);
        assert!(v.variance().is_none());
    }

    #[test]
    fn constant_input_has_zero_variance() {
        let mut v = EwVariance::<f32>::new(0.3);
        for _ in 0..200 {
            v.add(2.0);
        }
        assert!((v.mean().unwrap() - 2.0).abs() < 1e-6);
        assert!(v.variance().unwrap().abs() < 1e-6);
    }

    #[test]
    fn alternating_input_has_nonzero_variance() {
        let mut v = EwVariance::<f32>::new(0.3);
        for i in 0..200 {
            v.add(if i % 2 == 0 { 0.0 } else { 10.0 });
        }
        assert!(v.variance().unwrap() > 1.0);
    }

    #[test]
    fn responds_to_step_change_in_dispersion() {
        let mut v = EwVariance::<f32>::new(0.2);
        for _ in 0..100 {
            v.add(5.0);
        }
        let calm_var = v.variance().unwrap();

        for i in 0..100 {
            v.add(if i % 2 == 0 { 0.0 } else { 10.0 });
        }
        let volatile_var = v.variance().unwrap();

        assert!(calm_var < 1e-3);
        assert!(volatile_var > 5.0);
    }

    #[test]
    fn clear_resets() {
        let mut v = EwVariance::<f32>::new(0.5);
        v.add(1.0);
        v.add(2.0);
        v.clear();
        assert_eq!(v.count(), 0);
        assert!(v.mean().is_none());
        assert!(v.variance().is_none());
    }

    #[test]
    #[should_panic]
    fn rejects_alpha_zero() {
        let _ = EwVariance::<f32>::new(0.0);
    }

    #[test]
    #[should_panic]
    fn rejects_alpha_above_one() {
        let _ = EwVariance::<f32>::new(1.5);
    }
}
