use super::{DataSet, Loss};

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

    pub fn loss<F>(&self, mut error_fn: F) -> f32
    where
        F: FnMut(&Vec<f32>) -> Vec<f32>,
    {
        self.loss_function.calculate(&self.data_set, &mut error_fn)
    }
}
