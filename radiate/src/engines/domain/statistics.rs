use core::f32;

#[derive(PartialEq, Clone)]
pub struct Adder {
    compensation: f32,
    simple_sum: f32,
    sum: f32,
}

impl Adder {
    pub fn new() -> Self {
        Self {
            compensation: 0_f32,
            simple_sum: 0_f32,
            sum: 0_f32,
        }
    }

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
    pub fn new() -> Self {
        Self {
            m1: Adder::new(),
            m2: Adder::new(),
            m3: Adder::new(),
            m4: Adder::new(),
            sum: Adder::new(),
            count: 0,
            last_value: f32::NAN,
            max: f32::MIN,
            min: f32::MAX,
        }
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adder() {
        let mut adder = Adder::new();
        adder.add(1_f32);
        adder.add(2_f32);
        adder.add(3_f32);
        adder.add(4_f32);
        adder.add(5_f32);

        assert_eq!(adder.value(), 15_f32);
    }

    #[test]
    fn test_statistic() {
        let mut statistic = Statistic::new();
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