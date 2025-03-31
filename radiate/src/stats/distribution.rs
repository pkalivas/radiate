use super::P2Quantile;
use crate::Statistic;
use std::fmt::Debug;

#[derive(Clone, PartialEq)]
pub struct Distribution {
    pub statistic: Statistic,
    pub quantiles: [P2Quantile; 4], // 0.25, 0.5, 0.75, and 1.0 quantiles
    pub last_sequence: Vec<f32>,
}

impl Distribution {
    pub fn push(&mut self, value: f32) {
        self.statistic.add(value);
        self.last_sequence.push(value);
        for quantile in &mut self.quantiles {
            quantile.add(value);
        }
    }

    pub fn update(&mut self, value: &[f32]) {
        self.clear();
        for v in value {
            self.statistic.add(*v);
            self.last_sequence.push(*v);
            for quantile in &mut self.quantiles {
                quantile.add(*v);
            }
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

    pub fn first_quartile(&self) -> f32 {
        self.quantiles[0].value()
    }

    pub fn median(&self) -> f32 {
        self.quantiles[1].value()
    }

    pub fn third_quartile(&self) -> f32 {
        self.quantiles[2].value()
    }

    pub fn fourth_quartile(&self) -> f32 {
        self.quantiles[3].value()
    }

    pub fn clear(&mut self) {
        self.statistic.clear();
        self.last_sequence.clear();
        for quantile in &mut self.quantiles {
            quantile.clear();
        }
    }

    pub fn percentile(&self, p: f32) -> f32 {
        if !(0.0..=1.0).contains(&p) {
            panic!("Percentile must be between 0 and 100");
        }

        let count = self.last_sequence.len();
        if count == 0 {
            panic!("Cannot calculate percentile for an empty distribution");
        }

        // Sort the values.
        let mut sorted_values = self.last_sequence.clone();
        sorted_values.sort_by(|a, b| a.partial_cmp(b).unwrap());

        // Compute the fractional index using (n-1).
        let index = (count - 1) as f32 * p; // p should be in the range [0, 1]
        let lower = index.floor() as usize;
        let upper = index.ceil() as usize;

        if lower == upper {
            sorted_values[lower]
        } else {
            let weight = index - index.floor();
            sorted_values[lower] * (1.0 - weight) + sorted_values[upper] * weight
        }
    }
}

impl From<&[f32]> for Distribution {
    fn from(value: &[f32]) -> Self {
        let mut result = Distribution::default();
        result.update(value);
        result
    }
}

impl From<Vec<f32>> for Distribution {
    fn from(value: Vec<f32>) -> Self {
        let mut result = Distribution::default();
        result.update(&value);
        result
    }
}

impl Default for Distribution {
    fn default() -> Self {
        Distribution {
            statistic: Statistic::default(),
            quantiles: [
                P2Quantile::new(0.25),
                P2Quantile::new(0.5),
                P2Quantile::new(0.75),
                P2Quantile::new(1.0),
            ],
            last_sequence: Vec::new(),
        }
    }
}

impl Debug for Distribution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Distribution")
            .field("count", &self.count())
            .field("mean", &self.mean())
            .field("variance", &self.variance())
            .field("std_dev", &self.standard_deviation())
            .field("min", &self.min())
            .field("max", &self.max())
            .field("skewness", &self.skewness())
            .field("kurtosis", &self.kurtosis())
            .field("first_quartile", &self.first_quartile())
            .field("median", &self.median())
            .field("third_quartile", &self.third_quartile())
            .field("fourth_quartile", &self.fourth_quartile())
            .finish()
    }
}
