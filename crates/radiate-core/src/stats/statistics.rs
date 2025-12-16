use core::f32;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Adder {
    compensation: f32,
    simple_sum: f32,
    sum: f32,
}

impl Adder {
    pub fn value(&self) -> f32 {
        let result = self.sum + self.compensation;
        if result.is_nan() {
            self.simple_sum
        } else {
            result
        }
    }

    pub fn add(&mut self, value: f32) {
        let y = value - self.compensation;
        let t = self.sum + y;

        self.compensation = (t - self.sum) - y;
        self.sum = t;
        self.simple_sum += value;
    }
}

impl Default for Adder {
    fn default() -> Self {
        Adder {
            compensation: 0_f32,
            simple_sum: 0_f32,
            sum: 0_f32,
        }
    }
}

#[derive(PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Statistic {
    m1: Adder,
    m2: Adder,
    m3: Adder,
    m4: Adder,
    sum: Adder,
    count: i32,
    last_value: f32,
    max: f32,
    min: f32,
}

impl Statistic {
    pub fn new(initial_val: f32) -> Self {
        let mut result = Statistic::default();
        result.add(initial_val);
        result
    }

    pub fn last_value(&self) -> f32 {
        self.last_value
    }

    pub fn count(&self) -> i32 {
        self.count
    }

    pub fn min(&self) -> f32 {
        self.min
    }

    pub fn max(&self) -> f32 {
        self.max
    }

    pub fn mean(&self) -> f32 {
        if self.count == 0 {
            0_f32
        } else {
            self.m1.value()
        }
    }

    pub fn sum(&self) -> f32 {
        self.sum.value()
    }

    #[inline(always)]
    pub fn variance(&self) -> f32 {
        let mut value = f32::NAN;
        if self.count == 1 {
            value = self.m2.value();
        } else if self.count > 1 {
            value = self.m2.value() / (self.count - 1) as f32;
        }

        value
    }

    #[inline(always)]
    pub fn std_dev(&self) -> f32 {
        self.variance().sqrt()
    }

    #[inline(always)]
    pub fn skewness(&self) -> f32 {
        let mut value = f32::NAN;
        if self.count >= 3 {
            let temp = self.m2.value() / self.count as f32 - 1_f32;
            if temp < 10e-10_f32 {
                value = 0_f32;
            } else {
                value = self.count as f32 * self.m3.value()
                    / ((self.count as f32 - 1_f32)
                        * (self.count as f32 - 2_f32)
                        * temp.sqrt()
                        * temp)
            }
        }

        value
    }

    #[inline(always)]
    pub fn kurtosis(&self) -> f32 {
        let mut value = f32::NAN;
        if self.count >= 4 {
            let temp = self.m2.value() / self.count as f32 - 1_f32;
            if temp < 10e-10_f32 {
                value = 0_f32;
            } else {
                value = self.count as f32 * (self.count as f32 + 1_f32) * self.m4.value()
                    / ((self.count as f32 - 1_f32)
                        * (self.count as f32 - 2_f32)
                        * (self.count as f32 - 3_f32)
                        * temp
                        * temp)
            }
        }

        value
    }

    #[inline(always)]
    pub fn add(&mut self, value: f32) {
        self.count += 1;

        let n = self.count as f32;
        let d = value - self.m1.value();
        let dn = d / n;
        let dn2 = dn * dn;
        let t1 = d * dn * (n - 1_f32);

        self.m1.add(dn);

        self.m4.add(t1 * dn2 * (n * n - 3_f32 * n + 3_f32));
        self.m4
            .add(6_f32 * dn2 * self.m2.value() - 4_f32 * dn * self.m3.value());

        self.m3
            .add(t1 * dn * (n - 2_f32) - 3_f32 * dn * self.m2.value());
        self.m2.add(t1);

        self.last_value = value;
        self.max = if value > self.max { value } else { self.max };
        self.min = if value < self.min { value } else { self.min };
        self.sum.add(value);
    }

    pub fn clear(&mut self) {
        self.m1 = Adder::default();
        self.m2 = Adder::default();
        self.m3 = Adder::default();
        self.m4 = Adder::default();
        self.sum = Adder::default();
        self.count = 0;
        self.last_value = 0_f32;
        self.max = f32::MIN;
        self.min = f32::MAX;
    }

    pub fn merge(&mut self, other: &Statistic) {
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
        let n1 = self.count as f64;
        let n2 = other.count as f64;

        let mean1 = self.m1.value() as f64;
        let mean2 = other.m1.value() as f64;

        let m21 = self.m2.value() as f64;
        let m22 = other.m2.value() as f64;
        let m31 = self.m3.value() as f64;
        let m32 = other.m3.value() as f64;
        let m41 = self.m4.value() as f64;
        let m42 = other.m4.value() as f64;

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
            + 3.0 * delta * (n1 * m22 - n2 * m21) / n;

        let m4 = m41
            + m42
            + delta4 * n1n2 * (n1 * n1 - n1 * n2 + n2 * n2) / (n * n * n)
            + 6.0 * delta2 * (n1 * n1 * m22 + n2 * n2 * m21) / (n * n)
            + 4.0 * delta * (n1 * m32 - n2 * m31) / n;

        // Write back into Kahan adders.
        // Using `Adder::default()` + single `add` is fine:
        self.m1 = Adder::default();
        self.m1.add(mean as f32);

        self.m2 = Adder::default();
        self.m2.add(m2 as f32);

        self.m3 = Adder::default();
        self.m3.add(m3 as f32);

        self.m4 = Adder::default();
        self.m4.add(m4 as f32);

        // Merge auxiliary stats
        self.sum.add(other.sum()); // preserves Kahan accuracy for the total sum
        self.count += other.count;
        self.max = self.max.max(other.max);
        self.min = self.min.min(other.min);

        // "last_value" is a bit semantic; assuming `other` is later in time:
        self.last_value = other.last_value;
    }

    /// Convenience: return a merged copy instead of mutating in-place
    pub fn merged(mut self, other: &Statistic) -> Statistic {
        self.merge(other);
        self
    }
}

impl Default for Statistic {
    fn default() -> Self {
        Statistic {
            m1: Adder::default(),
            m2: Adder::default(),
            m3: Adder::default(),
            m4: Adder::default(),
            sum: Adder::default(),
            count: 0,
            last_value: 0_f32,
            max: f32::MIN,
            min: f32::MAX,
        }
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
        let mut statistic = Statistic::default();
        statistic.add(1_f32);
        statistic.add(2_f32);
        statistic.add(3_f32);
        statistic.add(4_f32);
        statistic.add(5_f32);

        assert_eq!(statistic.mean(), 3_f32);
        assert_eq!(statistic.variance(), 2.5_f32);
        assert_eq!(statistic.std_dev(), 1.5811388_f32);
        assert_eq!(statistic.skewness(), 0_f32);
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
        assert_eq!(merged.variance(), 3.5_f32);
        assert_eq!(merged.std_dev(), 1.8708287_f32);
        assert_eq!(merged.skewness(), 0_f32);
        assert_eq!(merged.count(), 6);
        assert_eq!(merged.min(), 1_f32);
        assert_eq!(merged.max(), 6_f32);
        assert_eq!(merged.sum(), 21_f32);
    }
}
