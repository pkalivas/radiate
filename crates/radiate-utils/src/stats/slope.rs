use crate::{Float, stats::statistics::Adder};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Slope<F: Float> {
    sum_y: Adder<F>,
    sum_xy: Adder<F>,
    count: u32,
}

impl<F: Float> Slope<F> {
    pub fn new() -> Self {
        Self {
            sum_y: Adder::<F>::default(),
            sum_xy: Adder::<F>::default(),
            count: 0,
        }
    }

    pub fn count(&self) -> u32 {
        self.count
    }

    pub fn add(&mut self, value: F) {
        let x = F::from(self.count).unwrap_or(F::ZERO);
        self.sum_y.add(value);
        self.sum_xy.add(x * value);
        self.count += 1;
    }

    pub fn value(&self) -> Option<F> {
        if self.count < 2 {
            return None;
        }

        let n = F::from(self.count)?;
        let one = F::ONE;
        let two = F::from(2.0)?;
        let six = F::from(6.0)?;

        let sum_x = n * (n - one) / two;
        let sum_x2 = n * (n - one) * (two * n - one) / six;

        let sum_y = self.sum_y.value();
        let sum_xy = self.sum_xy.value();

        let numerator = n * sum_xy - sum_x * sum_y;
        let denominator = n * sum_x2 - sum_x * sum_x;

        if denominator.abs() < F::EPS {
            None
        } else {
            Some(numerator / denominator)
        }
    }

    pub fn clear(&mut self) {
        self.sum_y = Adder::default();
        self.sum_xy = Adder::default();
        self.count = 0;
    }
}

impl<F: Float> Extend<F> for Slope<F> {
    fn extend<T: IntoIterator<Item = F>>(&mut self, iter: T) {
        for value in iter {
            self.add(value);
        }
    }
}

impl<F: Float> FromIterator<F> for Slope<F> {
    fn from_iter<T: IntoIterator<Item = F>>(iter: T) -> Self {
        let mut slope = Slope::new();
        slope.extend(iter);
        slope
    }
}

impl<F: Float> Default for Slope<F> {
    fn default() -> Self {
        Self::new()
    }
}

impl<F: Float> std::fmt::Debug for Slope<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Slope")
            .field("sum_y", &self.sum_y.value())
            .field("sum_xy", &self.sum_xy.value())
            .field("count", &self.count)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slope_increasing_line() {
        let mut slope = Slope::<f32>::new();
        slope.add(1.0);
        slope.add(2.0);
        slope.add(3.0);
        slope.add(4.0);

        let value = slope.value().unwrap();
        assert!((value - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_slope_flat_line() {
        let mut slope = Slope::<f32>::new();
        slope.add(5.0);
        slope.add(5.0);
        slope.add(5.0);
        slope.add(5.0);

        let value = slope.value().unwrap();
        assert!(value.abs() < 1e-6);
    }

    #[test]
    fn test_slope_decreasing_line() {
        let mut slope = Slope::<f32>::new();
        slope.add(4.0);
        slope.add(3.0);
        slope.add(2.0);
        slope.add(1.0);

        let value = slope.value().unwrap();
        assert!((value + 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_slope_two_points() {
        let mut slope = Slope::<f32>::new();
        slope.add(2.0);
        slope.add(6.0);

        let value = slope.value().unwrap();
        assert!((value - 4.0).abs() < 1e-6);
    }
}
