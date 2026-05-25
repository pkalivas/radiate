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

    pub fn log2(&self) -> f32 {
        (self.values.len() as f32).log2()
    }

    // #[inline(always)]
    // pub fn percentile(&self, p: F) -> F {
    //     if !(F::zero()..=F::from(100.0).unwrap()).contains(&p) {
    //         panic!("Percentile must be between 0 and 100");
    //     }

    //     let count = F::from(self.values.len()).unwrap();
    //     if count == F::zero() {
    //         return F::zero();
    //     }

    //     let index = (p / F::from(100.0).unwrap()) * count;

    //     let sorted_values = { &self.last_sequence };

    //     let index = index as usize;

    //     if index == 0 && !sorted_values.is_empty() {
    //         return sorted_values[0];
    //     } else if index == sorted_values.len() {
    //         return sorted_values[sorted_values.len() - 1];
    //     } else if index >= sorted_values.len() {
    //         panic!(
    //             "Index out of bounds for the sorted values {} >= {}",
    //             index,
    //             sorted_values.len()
    //         );
    //     }

    //     sorted_values[index]
    // }
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

// use crate::Statistic;
// #[cfg(feature = "serde")]
// use serde::{Deserialize, Serialize};
// use std::collections::HashMap;

// #[derive(Clone, PartialEq, Default)]
// #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
// pub struct Distribution {
//     pub statistic: Statistic,
//     pub last_sequence: Vec<f32>,
// }

// impl Distribution {
//     #[inline(always)]
//     pub fn add(&mut self, value: &[f32]) {
//         self.clear();
//         for v in value {
//             self.statistic.add(*v);
//             self.last_sequence.push(*v);
//         }
//     }

//     pub fn last_sequence(&self) -> &[f32] {
//         &self.last_sequence
//     }

//     pub fn count(&self) -> i32 {
//         self.last_sequence.len() as i32
//     }

//     pub fn mean(&self) -> f32 {
//         self.statistic.mean()
//     }

//     pub fn variance(&self) -> f32 {
//         self.statistic.variance().unwrap()
//     }

//     pub fn standard_deviation(&self) -> f32 {
//         self.statistic.std_dev().unwrap()
//     }

//     pub fn skewness(&self) -> f32 {
//         self.statistic.skewness().unwrap()
//     }

//     pub fn kurtosis(&self) -> f32 {
//         self.statistic.kurtosis().unwrap()
//     }

//     pub fn min(&self) -> f32 {
//         self.statistic.min()
//     }

//     pub fn max(&self) -> f32 {
//         self.statistic.max()
//     }

//     pub fn clear(&mut self) {
//         self.statistic.clear();
//         self.last_sequence.clear();
//     }

//     pub fn log2(&self) -> f32 {
//         (self.last_sequence().len() as f32).log2()
//     }

//     #[inline(always)]
//     pub fn entropy(&self) -> f32 {
//         let bin_width = 0.01;
//         let mut counts = HashMap::new();

//         for &value in &self.last_sequence {
//             let bin = (value / bin_width).floor();
//             *counts.entry(bin as i32).or_insert(0) += 1;
//         }

//         let total = self.last_sequence.len() as f32;
//         if total == 0.0 {
//             return 0.0;
//         }

//         counts
//             .values()
//             .map(|&count| {
//                 let p = count as f32 / total;
//                 -p * p.log2()
//             })
//             .sum()
//     }

//     #[inline(always)]
//     pub fn percentile(&self, p: f32) -> f32 {
//         if !(0.0..=100.0).contains(&p) {
//             panic!("Percentile must be between 0 and 100");
//         }

//         let count = self.last_sequence.len() as f32;
//         if count == 0.0 {
//             return 0.0;
//         }

//         let index = (p / 100.0) * count;

//         let sorted_values = { &self.last_sequence };

//         let index = index as usize;

//         if index == 0 && !sorted_values.is_empty() {
//             return sorted_values[0];
//         } else if index == sorted_values.len() {
//             return sorted_values[sorted_values.len() - 1];
//         } else if index >= sorted_values.len() {
//             panic!(
//                 "Index out of bounds for the sorted values {} >= {}",
//                 index,
//                 sorted_values.len()
//             );
//         }

//         sorted_values[index]
//     }
// }

// impl From<&[f32]> for Distribution {
//     fn from(value: &[f32]) -> Self {
//         let mut result = Distribution::default();
//         result.add(value);
//         result
//     }
// }

// impl From<Vec<f32>> for Distribution {
//     fn from(value: Vec<f32>) -> Self {
//         let mut result = Distribution::default();
//         result.add(&value);
//         result
//     }
// }

// impl From<&Vec<usize>> for Distribution {
//     fn from(value: &Vec<usize>) -> Self {
//         let mut dist = Distribution::default();
//         for v in value.iter().map(|&v| v as f32) {
//             dist.statistic.add(v);
//             dist.last_sequence.push(v);
//         }

//         dist
//     }
// }
