use num_traits::cast::FromPrimitive;
use num_traits::float::Float;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, Sub, SubAssign};

use super::SampleSet;

pub enum ErrorFunction {
    MSE,
    MAE,
    CrossEntropy,
    Diff,
}

impl ErrorFunction {
    pub fn calculate<T, F>(&self, samples: &SampleSet<T>, eval_func: &mut F) -> T
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

                    for i in 0..sample.2.len() {
                        let diff = sample.2[i].clone() - output[i].clone();
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
                        let diff = sample.2[i].clone() - output[i].clone();
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
                        sum += sample.2[i].clone() * output[i].clone().ln();
                    }
                }

                sum
            }
            ErrorFunction::Diff => {
                let mut sum = T::default();
                for sample in samples.get_samples().iter() {
                    let output = eval_func(&sample.1);

                    for i in 0..sample.2.len() {
                        sum += (sample.2[i].clone() - output[i].clone()).abs();
                    }
                }

                sum
            }
        }
    }
}
