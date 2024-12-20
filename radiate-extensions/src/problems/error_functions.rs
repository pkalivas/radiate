use num_traits::cast::FromPrimitive;
use num_traits::float::Float;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, Sub, SubAssign};

use super::DataSet;

pub enum ErrorFunction {
    MSE,
    MAE,
    CrossEntropy,
    Diff,
}

impl ErrorFunction {
    pub fn calculate<T, F>(&self, samples: &DataSet<T>, eval_func: &mut F) -> T
    where
        T: Clone
            + PartialEq
            + Default
            + Add<Output = T>
            + Div<Output = T>
            + Sub<Output = T>
            + Mul<Output = T>
            + Div<Output = T>
            + AddAssign
            + SubAssign
            + DivAssign
            + Float
            + FromPrimitive
            + DivAssign,
        F: FnMut(&Vec<T>) -> Vec<T>,
    {
        match self {
            ErrorFunction::MSE => {
                let mut sum = T::default();
                for sample in samples.get_samples().iter() {
                    let output = eval_func(&sample.1);

                    for (i, val) in output.iter().enumerate() {
                        let diff = sample.2[i] - *val;
                        sum += diff * diff;
                    }
                }

                sum / T::from_usize(samples.get_samples().len()).unwrap()
            }
            ErrorFunction::MAE => {
                let mut sum = T::default();
                for sample in samples.get_samples().iter() {
                    let output = eval_func(&sample.1);

                    for i in 0..sample.2.len() {
                        let diff = sample.2[i] - output[i];
                        sum += diff;
                    }
                }

                sum /= T::from_usize(samples.get_samples().len()).unwrap();
                sum
            }
            ErrorFunction::CrossEntropy => {
                let mut sum = T::default();
                for sample in samples.get_samples().iter() {
                    let output = eval_func(&sample.1);

                    for i in 0..sample.2.len() {
                        sum += sample.2[i] * output[i].ln();
                    }
                }

                sum
            }
            ErrorFunction::Diff => {
                let mut sum = T::default();
                for sample in samples.get_samples().iter() {
                    let output = eval_func(&sample.1);

                    for i in 0..sample.2.len() {
                        sum += (sample.2[i] - output[i]).abs();
                    }
                }

                sum
            }
        }
    }
}
