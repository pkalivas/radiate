use super::DataSet;

const ZERO: f32 = 0.0;

#[derive(Debug, Clone, Copy)]
pub enum Loss {
    MSE,
    MAE,
    CrossEntropy,
    Diff,
}

impl Loss {
    pub fn calculate<F>(&self, samples: &DataSet, eval_func: &mut F) -> f32
    where
        F: FnMut(&Vec<f32>) -> Vec<f32>,
    {
        let len = samples.len() as f32;

        match self {
            Loss::MSE => {
                let mut sum = ZERO;
                for sample in samples.iter() {
                    let output = eval_func(sample.input());
                    for (y_true, y_pred) in sample.output().iter().zip(output.iter()) {
                        let diff = y_true - y_pred;
                        sum += diff * diff;
                    }
                }
                sum / len
            }
            Loss::MAE => {
                let mut sum = ZERO;
                for sample in samples.iter() {
                    let output = eval_func(sample.input());

                    for i in 0..sample.output().len() {
                        let diff = sample.output()[i] - output[i];
                        sum += diff;
                    }
                }

                sum /= samples.iter().len() as f32;
                sum
            }
            Loss::CrossEntropy => {
                let mut sum = ZERO;
                for sample in samples.iter() {
                    let output = eval_func(sample.input());

                    for i in 0..sample.output().len() {
                        sum += sample.output()[i] * output[i].ln();
                    }
                }

                sum
            }
            Loss::Diff => {
                let mut sum = ZERO;
                for sample in samples.iter() {
                    let output = eval_func(sample.input());

                    for i in 0..sample.output().len() {
                        sum += (sample.output()[i] - output[i]).abs();
                    }
                }

                sum
            }
        }
    }
}
