use core::f32;

#[derive(PartialEq, Clone)]
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

    pub fn variance(&self) -> f32 {
        let mut value = f32::NAN;
        if self.count == 1 {
            value = self.m2.value();
        } else if self.count > 1 {
            value = self.m2.value() / (self.count - 1) as f32;
        }

        value
    }

    pub fn std_dev(&self) -> f32 {
        self.variance().sqrt()
    }

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
}
