use super::{data::Row, DataSet, Loss};

pub struct Regression {
    data_set: DataSet,
    loss_function: Loss,
}

impl Regression {
    pub fn new(sample_set: DataSet, loss_function: Loss) -> Self {
        Regression {
            data_set: sample_set,
            loss_function,
        }
    }

    pub fn error<F>(&self, mut error_fn: F) -> f32
    where
        F: FnMut(&Vec<f32>) -> Vec<f32>,
    {
        self.loss_function.calculate(&self.data_set, &mut error_fn)
    }

    pub fn iter(&self) -> &[Row] {
        self.data_set.iter()
    }
}
