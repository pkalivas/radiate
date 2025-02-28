use crate::Statistic;

#[derive(Clone, PartialEq, Default)]
pub struct Distribution {
    pub statistic: Statistic,
    pub last_sequence: Vec<f32>,
}

impl Distribution {
    pub fn add(&mut self, value: &[f32]) {
        self.clear();
        for v in value {
            self.statistic.add(*v);
            self.last_sequence.push(*v);
        }
    }

    pub fn last_sequence(&self) -> &Vec<f32> {
        &self.last_sequence
    }

    pub fn count(&self) -> i32 {
        self.statistic.count()
    }

    pub fn mean(&self) -> f32 {
        self.statistic.mean()
    }

    pub fn variance(&self) -> f32 {
        self.statistic.variance()
    }

    pub fn standard_deviation(&self) -> f32 {
        self.statistic.std_dev()
    }

    pub fn skewness(&self) -> f32 {
        self.statistic.skewness()
    }

    pub fn kurtosis(&self) -> f32 {
        self.statistic.kurtosis()
    }

    pub fn min(&self) -> f32 {
        self.statistic.min()
    }

    pub fn max(&self) -> f32 {
        self.statistic.max()
    }

    pub fn clear(&mut self) {
        self.statistic.clear();
        self.last_sequence.clear();
    }
}

impl Into<Distribution> for &[f32] {
    fn into(self) -> Distribution {
        let mut result = Distribution::default();
        result.add(self);
        result
    }
}

impl Into<Distribution> for Vec<f32> {
    fn into(self) -> Distribution {
        let mut result = Distribution::default();
        result.add(&self);
        result
    }
}
