use super::DataSet;

const ZERO: f32 = 0.0;

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
        match self {
            Loss::MSE => {
                let mut sum = ZERO;
                for sample in samples.iter() {
                    let output = eval_func(&sample.1);

                    for (i, val) in output.into_iter().enumerate() {
                        let diff = sample.2[i] - val;
                        sum += diff * diff;
                    }
                }

                sum / (samples.iter().len() as f32)
            }
            Loss::MAE => {
                let mut sum = ZERO;
                for sample in samples.iter() {
                    let output = eval_func(&sample.1);

                    for i in 0..sample.2.len() {
                        let diff = sample.2[i] - output[i];
                        sum += diff;
                    }
                }

                sum /= samples.iter().len() as f32;
                sum
            }
            Loss::CrossEntropy => {
                let mut sum = ZERO;
                for sample in samples.iter() {
                    let output = eval_func(&sample.1);

                    for i in 0..sample.2.len() {
                        sum += sample.2[i] * output[i].ln();
                    }
                }

                sum
            }
            Loss::Diff => {
                let mut sum = ZERO;
                for sample in samples.iter() {
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
