use crate::Float;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

/// Running quantile estimator using the P² algorithm.
///
/// Maintains an estimate of a single quantile in constant memory (five markers)
/// and constant per-update time. No samples are stored. Accuracy improves with
/// sample count and is generally good for unimodal distributions; for heavy-
/// tailed or multimodal data, exact methods over a buffered window will be
/// more accurate.
///
/// `q` must be in `(0, 1)` — use `min` / `max` from [`Statistic`](crate::Statistic)
/// for the endpoints.
///
/// Reference: Jain & Chlamtac, "The P² Algorithm for Dynamic Calculation of
/// Quantiles and Histograms Without Storing Observations" (1985).
#[derive(Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Quantile<F: Float = f32> {
    q: F,
    heights: [F; 5],
    positions: [F; 5],
    desired: [F; 5],
    increments: [F; 5],
    count: u32,
}

impl<F: Float> Quantile<F> {
    pub fn new(q: F) -> Self {
        assert!(
            q > F::ZERO && q < F::ONE,
            "Quantile q must be in the open interval (0, 1)"
        );
        Self {
            q,
            heights: [F::ZERO; 5],
            positions: [F::ZERO; 5],
            desired: [F::ZERO; 5],
            increments: Self::compute_increments(q),
            count: 0,
        }
    }

    pub fn q(&self) -> F {
        self.q
    }

    pub fn count(&self) -> u32 {
        self.count
    }

    /// Current quantile estimate. Returns `None` before any sample has been added.
    /// For fewer than five samples, falls back to exact linear interpolation over
    /// the buffered values; after five samples, returns the P² estimate.
    pub fn value(&self) -> Option<F> {
        match self.count {
            0 => None,
            n if n < 5 => Some(self.interp_partial(n as usize)),
            _ => Some(self.heights[2]),
        }
    }

    pub fn add(&mut self, x: F) {
        if self.count < 5 {
            self.heights[self.count as usize] = x;
            self.count += 1;
            if self.count == 5 {
                self.heights
                    .sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
                for i in 0..5 {
                    self.positions[i] = F::from(i + 1).unwrap_or(F::ZERO);
                }
                self.desired[0] = F::ONE;
                self.desired[1] = F::ONE + F::TWO * self.q;
                self.desired[2] = F::ONE + F::FOUR * self.q;
                self.desired[3] = F::THREE + F::TWO * self.q;
                self.desired[4] = F::FIVE;
            }
            return;
        }

        let k = self.find_cell(x);

        for i in (k + 1)..5 {
            self.positions[i] = self.positions[i] + F::ONE;
        }

        for i in 0..5 {
            self.desired[i] = self.desired[i] + self.increments[i];
        }

        for i in 1..4 {
            let d = self.desired[i] - self.positions[i];
            let up = self.positions[i + 1] - self.positions[i];
            let down = self.positions[i - 1] - self.positions[i];

            let sign = if d >= F::ONE && up > F::ONE {
                F::ONE
            } else if d <= -F::ONE && down < -F::ONE {
                -F::ONE
            } else {
                continue;
            };

            let qs = self.parabolic(i, sign);
            let new_h = if self.heights[i - 1] < qs && qs < self.heights[i + 1] {
                qs
            } else {
                self.linear(i, sign)
            };
            self.heights[i] = new_h;
            self.positions[i] = self.positions[i] + sign;
        }

        self.count += 1;
    }

    pub fn clear(&mut self) {
        self.heights = [F::ZERO; 5];
        self.positions = [F::ZERO; 5];
        self.desired = [F::ZERO; 5];
        self.increments = Self::compute_increments(self.q);
        self.count = 0;
    }

    fn compute_increments(q: F) -> [F; 5] {
        let half = F::from(0.5).unwrap_or(F::ZERO);
        [F::ZERO, q * half, q, (F::ONE + q) * half, F::ONE]
    }

    fn find_cell(&mut self, x: F) -> usize {
        if x < self.heights[0] {
            self.heights[0] = x;
            return 0;
        }
        if x >= self.heights[4] {
            self.heights[4] = x;
            return 3;
        }
        let mut k = 0;
        while k < 3 && x >= self.heights[k + 1] {
            k += 1;
        }
        k
    }

    fn parabolic(&self, i: usize, d: F) -> F {
        let n_prev = self.positions[i - 1];
        let n = self.positions[i];
        let n_next = self.positions[i + 1];
        let h_prev = self.heights[i - 1];
        let h = self.heights[i];
        let h_next = self.heights[i + 1];

        h + d / (n_next - n_prev)
            * ((n - n_prev + d) * (h_next - h) / (n_next - n)
                + (n_next - n - d) * (h - h_prev) / (n - n_prev))
    }

    fn linear(&self, i: usize, d: F) -> F {
        let neighbor = if d > F::ZERO { i + 1 } else { i - 1 };
        self.heights[i]
            + d * (self.heights[neighbor] - self.heights[i])
                / (self.positions[neighbor] - self.positions[i])
    }

    fn interp_partial(&self, n: usize) -> F {
        let mut buf = [F::ZERO; 5];
        buf[..n].copy_from_slice(&self.heights[..n]);
        buf[..n].sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));

        if n == 1 {
            return buf[0];
        }
        let q_f = self.q.to_f64().unwrap_or(0.0);
        let pos = q_f * (n - 1) as f64;
        let lo = pos.floor() as usize;
        let hi = pos.ceil() as usize;
        let frac = F::from(pos - lo as f64).unwrap_or(F::ZERO);
        buf[lo] + frac * (buf[hi] - buf[lo])
    }
}

impl<F: Float> Extend<F> for Quantile<F> {
    fn extend<T: IntoIterator<Item = F>>(&mut self, iter: T) {
        for v in iter {
            self.add(v);
        }
    }
}

impl<F: Float> std::fmt::Debug for Quantile<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Quantile")
            .field("q", &self.q)
            .field("count", &self.count)
            .field("value", &self.value())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn median_of_uniform_sequence() {
        let mut q = Quantile::<f32>::new(0.5);
        for i in 1..=100 {
            q.add(i as f32);
        }
        let v = q.value().unwrap();
        // P² is approximate; tolerate a few percent off true median (~50.5)
        assert!((v - 50.5).abs() < 2.0, "got {v}");
    }

    #[test]
    fn p95_of_uniform_sequence() {
        let mut q = Quantile::<f32>::new(0.95);
        for i in 1..=1000 {
            q.add(i as f32);
        }
        let v = q.value().unwrap();
        assert!((v - 950.0).abs() < 15.0, "got {v}");
    }

    #[test]
    fn constant_input_returns_constant() {
        let mut q = Quantile::<f32>::new(0.5);
        for _ in 0..50 {
            q.add(7.0);
        }
        assert_eq!(q.value().unwrap(), 7.0);
    }

    #[test]
    fn single_sample_returns_that_sample() {
        let mut q = Quantile::<f32>::new(0.5);
        q.add(3.14);
        assert_eq!(q.value().unwrap(), 3.14);
    }

    #[test]
    fn empty_returns_none() {
        let q = Quantile::<f32>::new(0.5);
        assert!(q.value().is_none());
    }

    #[test]
    fn fewer_than_five_uses_exact_interp() {
        let mut q = Quantile::<f32>::new(0.5);
        q.add(1.0);
        q.add(3.0);
        q.add(5.0);
        // exact median of [1,3,5] is 3
        assert_eq!(q.value().unwrap(), 3.0);
    }

    #[test]
    fn clear_resets_state() {
        let mut q = Quantile::<f32>::new(0.5);
        for i in 1..=20 {
            q.add(i as f32);
        }
        q.clear();
        assert_eq!(q.count(), 0);
        assert!(q.value().is_none());
    }

    #[test]
    fn extend_from_iter() {
        let mut q = Quantile::<f32>::new(0.5);
        q.extend((1..=100).map(|i| i as f32));
        assert_eq!(q.count(), 100);
        let v = q.value().unwrap();
        assert!((v - 50.5).abs() < 2.0);
    }

    #[test]
    #[should_panic]
    fn rejects_q_zero() {
        let _ = Quantile::<f32>::new(0.0);
    }

    #[test]
    #[should_panic]
    fn rejects_q_one() {
        let _ = Quantile::<f32>::new(1.0);
    }
}
