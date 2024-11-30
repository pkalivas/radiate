use super::data_set::Row;
use super::{data_set::DataSet, error_functions::ErrorFunction};
use num_traits::cast::FromPrimitive;
use num_traits::float::Float;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

pub struct Regression<T> {
    pub sample_set: DataSet<T>,
    pub loss_function: ErrorFunction,
}

impl<T> Regression<T> {
    pub fn new(sample_set: DataSet<T>, loss_function: ErrorFunction) -> Self {
        Regression {
            sample_set,
            loss_function,
        }
    }

    pub fn from(loss_function: ErrorFunction, samples: Vec<(Vec<T>, Vec<T>)>) -> Self {
        let mut sample_set = DataSet::new();
        for (input, output) in samples {
            sample_set.add_sample(input, output);
        }
        Regression {
            sample_set,
            loss_function,
        }
    }

    pub fn error<F>(&self, mut error_fn: F) -> T
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
            + MulAssign
            + Float
            + FromPrimitive,
        F: FnMut(&Vec<T>) -> Vec<T>,
    {
        self.loss_function
            .calculate(&self.sample_set, &mut error_fn)
    }

    pub fn get_samples(&self) -> &[Row<T>] {
        self.sample_set.get_samples()
    }

    pub fn get_loss_function(&self) -> &ErrorFunction {
        &self.loss_function
    }
}
