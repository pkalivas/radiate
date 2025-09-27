use super::DataSet;
use crate::EvalMut;

const ZERO: f32 = 0.0;

#[derive(Debug, Clone, Copy)]
pub enum Loss {
    MSE,
    MAE,
    CrossEntropy,
    Diff,
}

impl Loss {
    #[inline]
    pub fn calculate(&self, data_set: &DataSet, eval: &mut impl EvalMut<[f32], Vec<f32>>) -> f32 {
        let len = data_set.len() as f32;

        match self {
            Loss::MSE => {
                let sum = data_set
                    .iter()
                    .map(|sample| {
                        let output = eval.eval_mut(sample.input());
                        sample
                            .output()
                            .iter()
                            .zip(output.iter())
                            .map(|(y_true, y_pred)| {
                                let diff = y_true - y_pred;
                                diff * diff
                            })
                            .sum::<f32>()
                    })
                    .sum::<f32>();

                sum / len
            }
            Loss::MAE => {
                let mut sum = ZERO;
                for sample in data_set.iter() {
                    let output = eval.eval_mut(sample.input());

                    for i in 0..sample.output().len() {
                        let diff = sample.output()[i] - output[i];
                        sum += diff.abs();
                    }
                }

                sum /= data_set.iter().len() as f32;
                sum
            }
            Loss::CrossEntropy => {
                let mut sum = ZERO;
                for sample in data_set.iter() {
                    let output = eval.eval_mut(sample.input());

                    for i in 0..sample.output().len() {
                        sum += sample.output()[i] * output[i].ln();
                    }
                }

                sum
            }
            Loss::Diff => {
                let mut sum = ZERO;
                for sample in data_set.iter() {
                    let output = eval.eval_mut(sample.input());

                    for i in 0..sample.output().len() {
                        sum += (sample.output()[i] - output[i]).abs();
                    }
                }

                sum
            }
        }
    }
}
