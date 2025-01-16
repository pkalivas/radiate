const ZERO: f32 = 0.0;

pub struct Regression {
    data_set: DataSet,
    loss_function: ErrorFunction,
}

impl Regression {
    pub fn new(sample_set: DataSet, loss_function: ErrorFunction) -> Self {
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

#[derive(Debug, Clone, Default)]
pub struct Row(pub usize, pub Vec<f32>, pub Vec<f32>);

#[derive(Default)]
pub struct DataSet {
    samples: Vec<Row>,
}

impl DataSet {
    pub fn new(inputs: Vec<Vec<f32>>, outputs: Vec<Vec<f32>>) -> Self {
        let mut samples = Vec::new();
        for (input, output) in inputs.into_iter().zip(outputs.into_iter()) {
            samples.push(Row(samples.len(), input, output));
        }
        DataSet { samples }
    }

    pub fn iter(&self) -> &[Row] {
        &self.samples
    }

    pub fn iter_mut(&mut self) -> &mut [Row] {
        &mut self.samples
    }

    pub fn len(&self) -> usize {
        self.samples.len()
    }
}

pub enum ErrorFunction {
    MSE,
    MAE,
    CrossEntropy,
    Diff,
}

impl ErrorFunction {
    pub fn calculate<F>(&self, samples: &DataSet, eval_func: &mut F) -> f32
    where
        F: FnMut(&Vec<f32>) -> Vec<f32>,
    {
        match self {
            ErrorFunction::MSE => {
                let mut sum = ZERO;
                for sample in samples.iter().iter() {
                    let output = eval_func(&sample.1);

                    for (i, val) in output.into_iter().enumerate() {
                        let diff = sample.2[i] - val;
                        sum += diff * diff;
                    }
                }

                sum / (samples.iter().len() as f32)
            }
            ErrorFunction::MAE => {
                let mut sum = ZERO;
                for sample in samples.iter().iter() {
                    let output = eval_func(&sample.1);

                    for i in 0..sample.2.len() {
                        let diff = sample.2[i] - output[i];
                        sum += diff;
                    }
                }

                sum /= samples.iter().len() as f32;
                sum
            }
            ErrorFunction::CrossEntropy => {
                let mut sum = ZERO;
                for sample in samples.iter().iter() {
                    let output = eval_func(&sample.1);

                    for i in 0..sample.2.len() {
                        sum += sample.2[i] * output[i].ln();
                    }
                }

                sum
            }
            ErrorFunction::Diff => {
                let mut sum = ZERO;
                for sample in samples.iter().iter() {
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
