use core::f32;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, hash::Hash};

use crate::{Float, Primitive};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(PartialEq, Clone)]
pub struct Adder<F: Float = f32> {
    compensation: F,
    simple_sum: F,
    sum: F,
}

impl<F: Float> Adder<F> {
    pub fn value(&self) -> F {
        let result = self.sum + self.compensation;
        if result.is_nan() {
            self.simple_sum
        } else {
            result
        }
    }

    pub fn add(&mut self, value: F) {
        let y = value - self.compensation;
        let t = self.sum + y;

        self.compensation = (t - self.sum) - y;
        self.sum = t;
        self.simple_sum = self.simple_sum + value;
    }
}

impl<F: Float> Default for Adder<F> {
    fn default() -> Self {
        Adder {
            compensation: F::ZERO,
            simple_sum: F::ZERO,
            sum: F::ZERO,
        }
    }
}

#[derive(PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Statistic<F: Float = f32> {
    m1: Adder<F>,
    m2: Adder<F>,
    m3: Adder<F>,
    m4: Adder<F>,
    sum: Adder<F>,
    count: i32,
    last_value: F,
    max: F,
    min: F,
}

impl<F: Float> Statistic<F> {
    pub fn new(initial_val: F) -> Self {
        let mut result = Statistic::default();
        result.add(initial_val);
        result
    }

    pub fn last_value(&self) -> F {
        self.last_value
    }

    pub fn count(&self) -> i32 {
        self.count
    }

    pub fn min(&self) -> F {
        self.min
    }

    pub fn max(&self) -> F {
        self.max
    }

    pub fn mean(&self) -> F {
        if self.count == 0 {
            F::ZERO
        } else {
            self.m1.value()
        }
    }

    pub fn sum(&self) -> F {
        self.sum.value()
    }

    #[inline(always)]
    pub fn variance(&self) -> Option<F> {
        let mut value = F::MIN;
        if self.count == 1 {
            value = self.m2.value();
        } else if self.count > 1 {
            value = self.m2.value() / (F::from(self.count)? - F::ONE);
        } else if self.count == 0 {
            return None;
        }

        Some(value)
    }

    #[inline(always)]
    pub fn std_dev(&self) -> Option<F> {
        Some(self.variance()?.sqrt())
    }

    #[inline(always)]
    pub fn skewness(&self) -> Option<F> {
        let mut value = F::NAN;
        let count = F::from(self.count)?;
        if self.count >= 3 {
            let temp = self.m2.value() / count - F::ONE;
            if temp < F::EPS {
                value = F::ZERO;
            } else {
                value = count * self.m3.value()
                    / ((count - F::ONE) * (count - F::TWO) * temp.sqrt() * temp)
            }
        }

        Some(value)
    }

    #[inline(always)]
    pub fn kurtosis(&self) -> Option<F> {
        let mut value = F::NAN;
        let count = F::from(self.count)?;

        if self.count >= 4 {
            let temp = self.m2.value() / count - F::ONE;
            if temp < F::EPS {
                value = F::ZERO;
            } else {
                value = count * (count + F::ONE) * self.m4.value()
                    / ((count - F::ONE) * (count - F::TWO) * (count - F::THREE) * temp * temp)
            }
        }

        Some(value)
    }

    #[inline(always)]
    pub fn add(&mut self, value: F) -> Option<()> {
        self.count += 1;

        let n = F::from(self.count)?;
        let d = value - self.m1.value();
        let dn = d / n;
        let dn2 = dn * dn;
        let t1 = d * dn * (n - F::ONE);

        self.m1.add(dn);

        self.m4.add(t1 * dn2 * (n * n - F::THREE * n + F::THREE));
        self.m4
            .add(F::SIX * dn2 * self.m2.value() - F::FOUR * dn * self.m3.value());

        self.m3
            .add(t1 * dn * (n - F::TWO) - F::THREE * dn * self.m2.value());
        self.m2.add(t1);

        self.last_value = value;
        self.max = if value > self.max { value } else { self.max };
        self.min = if value < self.min { value } else { self.min };
        self.sum.add(value);

        Some(())
    }

    pub fn clear(&mut self) {
        self.m1 = Adder::default();
        self.m2 = Adder::default();
        self.m3 = Adder::default();
        self.m4 = Adder::default();
        self.sum = Adder::default();
        self.count = 0;
        self.last_value = F::ZERO;
        self.max = F::MIN;
        self.min = F::MAX;
    }

    pub fn merge(&mut self, other: &Statistic<F>) {
        if other.count == 0 {
            return;
        }

        if self.count == 0 {
            *self = other.clone();
            return;
        }

        if other.count == 1 {
            self.add(other.last_value);
            return;
        }

        if self.count == 1 {
            let last_value = self.last_value;
            *self = other.clone();
            self.add(last_value);
            return;
        }

        // Use f64 for more accurate intermediate math
        let n1 = F::from(self.count).unwrap_or(F::ZERO);
        let n2 = F::from(other.count).unwrap_or(F::ZERO);

        let mean1 = self.m1.value();
        let mean2 = other.m1.value();

        let m21 = self.m2.value();
        let m22 = other.m2.value();
        let m31 = self.m3.value();
        let m32 = other.m3.value();
        let m41 = self.m4.value();
        let m42 = other.m4.value();

        let n = n1 + n2;
        let delta = mean2 - mean1;
        let delta2 = delta * delta;
        let delta3 = delta2 * delta;
        let delta4 = delta3 * delta;
        let n1n2 = n1 * n2;

        // Combined mean and moments (Pébay formulas)
        let mean = (n1 * mean1 + n2 * mean2) / n;

        let m2 = m21 + m22 + delta2 * n1n2 / n;

        let m3 = m31
            + m32
            + delta3 * n1n2 * (n1 - n2) / (n * n)
            + F::THREE * delta * (n1 * m22 - n2 * m21) / n;

        let m4 = m41
            + m42
            + delta4 * n1n2 * (n1 * n1 - n1 * n2 + n2 * n2) / (n * n * n)
            + F::SIX * delta2 * (n1 * n1 * m22 + n2 * n2 * m21) / (n * n)
            + F::FOUR * delta * (n1 * m32 - n2 * m31) / n;

        // Write back into Kahan adders.
        // Using `Adder::default()` + single `add` is fine:
        self.m1 = Adder::default();
        self.m1.add(mean);

        self.m2 = Adder::default();
        self.m2.add(m2);

        self.m3 = Adder::default();
        self.m3.add(m3);

        self.m4 = Adder::default();
        self.m4.add(m4);

        // Merge auxiliary stats
        self.sum.add(other.sum()); // preserves Kahan accuracy for the total sum
        self.count += other.count;
        self.max = self.max.max(other.max);
        self.min = self.min.min(other.min);

        // "last_value" is a bit semantic; assuming `other` is later in time:
        self.last_value = other.last_value;
    }

    /// Convenience: return a merged copy instead of mutating in-place
    pub fn merged(mut self, other: &Statistic<F>) -> Statistic<F> {
        self.merge(other);
        self
    }
}

impl<T: Primitive, F: Float> FromIterator<T> for Statistic<F> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut statistic = Statistic::<F>::default();
        for item in iter {
            if let Some(value) = item.extract::<F>() {
                statistic.add(value);
            }
        }
        statistic
    }
}

impl From<f32> for Statistic {
    fn from(value: f32) -> Self {
        Statistic::new(value)
    }
}

impl From<i32> for Statistic {
    fn from(value: i32) -> Self {
        Statistic::new(value as f32)
    }
}

impl From<usize> for Statistic {
    fn from(value: usize) -> Self {
        Statistic::new(value as f32)
    }
}

impl<F: Float> Default for Statistic<F> {
    fn default() -> Self {
        Statistic {
            m1: Adder::default(),
            m2: Adder::default(),
            m3: Adder::default(),
            m4: Adder::default(),
            sum: Adder::default(),
            count: 0,
            last_value: F::ZERO,
            max: F::MIN,
            min: F::MAX,
        }
    }
}

impl<F: Float> Hash for Statistic<F> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.count.hash(state);
        self.last_value.num_hash(state);
        self.max.num_hash(state);
        self.min.num_hash(state);
        self.sum.value().num_hash(state);
        self.m1.value().num_hash(state);
        self.m2.value().num_hash(state);
        self.m3.value().num_hash(state);
        self.m4.value().num_hash(state);
    }
}

impl<F: Debug + Float> Debug for Statistic<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Statistic")
            .field("count", &self.count)
            .field("last_value", &self.last_value)
            .field("max", &self.max)
            .field("min", &self.min)
            .field("sum", &self.sum.value())
            .field("mean", &self.mean())
            .field("variance", &self.variance())
            .field("std_dev", &self.std_dev())
            .field("skewness", &self.skewness())
            .field("kurtosis", &self.kurtosis())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adder() {
        let mut adder = Adder::default();
        adder.add(1_f32);
        adder.add(2_f32);
        adder.add(3_f32);
        adder.add(4_f32);
        adder.add(5_f32);

        assert_eq!(adder.value(), 15_f32);
    }

    #[test]
    fn test_statistic() {
        let mut statistic = Statistic::<f32>::default();
        statistic.add(1_f32);
        statistic.add(2_f32);
        statistic.add(3_f32);
        statistic.add(4_f32);
        statistic.add(5_f32);

        assert_eq!(statistic.mean(), 3_f32);
        assert_eq!(statistic.variance().unwrap(), 2.5_f32);
        assert_eq!(statistic.std_dev().unwrap(), 1.5811388_f32);
        assert_eq!(statistic.skewness().unwrap(), 0_f32);
    }

    #[test]
    fn test_statistic_merge() {
        let mut stat1 = Statistic::default();
        stat1.add(1_f32);
        stat1.add(2_f32);
        stat1.add(3_f32);

        let mut stat2 = Statistic::default();
        stat2.add(4_f32);
        stat2.add(5_f32);
        stat2.add(6_f32);

        let merged = stat1.merged(&stat2);
        assert_eq!(merged.mean(), 3.5_f32);
        assert_eq!(merged.variance().unwrap(), 3.5_f32);
        assert_eq!(merged.std_dev().unwrap(), 1.8708287_f32);
        assert_eq!(merged.skewness().unwrap(), 0_f32);
        assert_eq!(merged.count(), 6);
        assert_eq!(merged.min(), 1_f32);
        assert_eq!(merged.max(), 6_f32);
        assert_eq!(merged.sum(), 21_f32);
    }
}
