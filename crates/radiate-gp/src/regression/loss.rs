use super::DataSet;
use crate::{EvalMut, eval::EvalIntoMut};
use std::cell::RefCell;
use std::thread_local;

const ZERO: f32 = 0.0;

thread_local! {
    // one scratch buffer per thread
    static LOSS_SCRATCH: RefCell<Vec<f32>> = RefCell::new(Vec::new());
}

#[derive(Debug, Clone, Copy)]
pub enum Loss {
    MSE,
    MAE,
    CrossEntropy,
    Diff,
}

impl Loss {
    #[inline]
    pub fn calc_tls(&self, data_set: &DataSet, eval: &mut impl EvalIntoMut<[f32], [f32]>) -> f32 {
        let out_len = data_set.shape().2;

        LOSS_SCRATCH.with(|cell| {
            let mut buf = cell.borrow_mut();
            if buf.len() < out_len {
                buf.resize(out_len, 0.0);
            }
            // now reuse buf
            self.calc_with_buf(data_set, eval, &mut buf[..out_len])
        })
    }

    #[inline]
    fn calc_with_buf(
        &self,
        data_set: &DataSet,
        eval: &mut impl EvalIntoMut<[f32], [f32]>,
        buffer: &mut [f32],
    ) -> f32 {
        let n = data_set.len() as f32;
        match self {
            Loss::MSE => {
                let mut sum = 0.0;
                for sample in data_set.iter() {
                    eval.eval_into_mut(sample.input(), buffer);
                    let target = sample.output();
                    for i in 0..target.len() {
                        let d = target[i] - buffer[i];
                        sum += d * d;
                    }
                }
                sum / n
            }
            Loss::MAE => {
                let mut sum = 0.0;
                for sample in data_set.iter() {
                    eval.eval_into_mut(sample.input(), buffer);
                    let target = sample.output();
                    for i in 0..target.len() {
                        let d = target[i] - buffer[i];
                        sum += d.abs();
                    }
                }
                sum / n
            }
            Loss::CrossEntropy => {
                const EPS: f32 = 1e-7;
                let mut sum = 0.0;
                for sample in data_set.iter() {
                    eval.eval_into_mut(sample.input(), buffer);
                    let target = sample.output();
                    for i in 0..target.len() {
                        let p = target[i];
                        let q = buffer[i].clamp(EPS, 1.0);
                        sum += -p * q.ln();
                    }
                }
                sum / n
            }
            Loss::Diff => {
                let mut sum = 0.0;
                for sample in data_set.iter() {
                    eval.eval_into_mut(sample.input(), buffer);
                    let target = sample.output();
                    for i in 0..target.len() {
                        sum += (target[i] - buffer[i]).abs();
                    }
                }
                sum / n
            }
        }
    }

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
