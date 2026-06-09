use crate::{Float, Statistic};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Distribution<F: Float> {
    statistic: Statistic<F>,
    values: Vec<F>,
}

impl<F: Float> Distribution<F> {
    pub fn with_capacity(capacity: usize) -> Self {
        Distribution {
            statistic: Statistic::default(),
            values: Vec::with_capacity(capacity),
        }
    }

    #[inline(always)]
    pub fn add(&mut self, value: F) {
        if self.values.capacity() == self.values.len() {
            panic!(
                "Distribution capacity exceeded: {}. Consider increasing the capacity or using a different data structure.",
                self.values.capacity()
            );
        }

        self.statistic.add(value);
        self.values.push(value);
    }

    pub fn count(&self) -> usize {
        self.values.len()
    }

    pub fn mean(&self) -> F {
        self.statistic.mean()
    }

    pub fn variance(&self) -> Option<F> {
        self.statistic.variance()
    }

    pub fn standard_deviation(&self) -> Option<F> {
        self.statistic.std_dev()
    }

    pub fn skewness(&self) -> Option<F> {
        self.statistic.skewness()
    }

    pub fn kurtosis(&self) -> Option<F> {
        self.statistic.kurtosis()
    }

    pub fn min(&self) -> F {
        self.statistic.min()
    }

    pub fn max(&self) -> F {
        self.statistic.max()
    }

    pub fn clear(&mut self) {
        self.statistic.clear();
        self.values.clear();
    }
}

impl From<&[f32]> for Distribution<f32> {
    fn from(value: &[f32]) -> Self {
        let mut result = Distribution::with_capacity(value.len());
        for &v in value {
            result.add(v);
        }

        result
    }
}

impl From<Vec<f32>> for Distribution<f32> {
    fn from(value: Vec<f32>) -> Self {
        let mut result = Distribution::with_capacity(value.len());
        for v in value {
            result.add(v);
        }
        result
    }
}

impl From<&Vec<usize>> for Distribution<f32> {
    fn from(value: &Vec<usize>) -> Self {
        let mut dist = Distribution::with_capacity(value.len());
        for v in value.iter().map(|&v| v as f32) {
            dist.add(v);
        }

        dist
    }
}
