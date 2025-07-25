use crate::Statistic;

#[derive(Clone, PartialEq, Default)]
pub struct Distribution {
    pub statistic: Statistic,
    pub last_sequence: Vec<f32>,
}

impl Distribution {
    pub fn push(&mut self, value: f32) {
        self.statistic.add(value);
        self.last_sequence.push(value);
    }

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
        // self.statistic.clear();
        self.last_sequence.clear();
    }

    pub fn log2(&self) -> f32 {
        (self.last_sequence().len() as f32).log2()
    }

    pub fn entropy(&self) -> f32 {
        let bin_width = 0.01; // You can tune this for your resolution
        let mut counts = std::collections::HashMap::new();

        for &value in &self.last_sequence {
            let bin = (value / bin_width).floor();
            *counts.entry(bin as i32).or_insert(0usize) += 1;
        }

        let total = self.last_sequence.len() as f32;
        if total == 0.0 {
            return 0.0;
        }

        counts
            .values()
            .map(|&count| {
                let p = count as f32 / total;
                -p * p.log2()
            })
            .sum()
    }

    pub fn percentile(&self, p: f32) -> f32 {
        if p < 0.0 || p > 100.0 {
            panic!("Percentile must be between 0 and 100");
        }

        let count = self.last_sequence.len() as f32;
        if count == 0 as f32 {
            panic!("Cannot calculate percentile for an empty distribution");
        }
        let index = (p / 100.0) * count;
        let sorted_values = {
            let mut values = self.last_sequence.clone();
            values.sort_by(|a, b| a.partial_cmp(b).unwrap());
            values
        };

        let index = index as usize;
        if index >= sorted_values.len() {
            panic!("Index out of bounds for the sorted values");
        }

        sorted_values[index]
    }
}

impl From<&[f32]> for Distribution {
    fn from(value: &[f32]) -> Self {
        let mut result = Distribution::default();
        result.add(value);
        result
    }
}

impl From<Vec<f32>> for Distribution {
    fn from(value: Vec<f32>) -> Self {
        let mut result = Distribution::default();
        result.add(&value);
        result
    }
}

// pub fn entropy(&self, bin_count: usize) -> f32 {
//     if self.last_sequence.is_empty() || bin_count == 0 {
//         return 0.0;
//     }

//     let min = self.min();
//     let max = self.max();

//     if (max - min).abs() < std::f32::EPSILON {
//         return 0.0; // All values are (almost) the same
//     }

//     let bin_width = (max - min) / bin_count as f32;
//     let mut bins = vec![0usize; bin_count];

//     for &v in &self.last_sequence {
//         let bin_index = ((v - min) / bin_width).floor() as usize;
//         let clamped = bin_index.min(bin_count - 1);
//         bins[clamped] += 1;
//     }

//     let total = self.last_sequence.len() as f32;
//     bins.iter()
//         .filter(|&&count| count > 0)
//         .map(|&count| {
//             let p = count as f32 / total;
//             -p * p.log2()
//         })
//         .sum()
// }
