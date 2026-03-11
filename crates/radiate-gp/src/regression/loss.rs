use super::DataSet;
use crate::EvalMut;

const EPS: f32 = 1e-7;

#[derive(Debug, Clone, Copy)]
pub enum Loss {
    MSE,
    MAE,
    XEnt,
    Diff,
}

impl Loss {
    #[inline]
    pub fn calc(&self, data_set: &DataSet<f32>, eval: &mut impl EvalMut<[f32], Vec<f32>>) -> f32 {
        let out_len = data_set.shape().2;
        let mut buffer = vec![0.0; out_len];

        self.calculate(data_set, &mut buffer[..out_len], |x, y| {
            let v = eval.eval_mut(x);
            y.copy_from_slice(&v);
        })
    }

    #[inline]
    pub fn calculate<F>(
        &self,
        data_set: &DataSet<f32>,
        buffer: &mut [f32],
        mut eval_into_buf: F,
    ) -> f32
    where
        F: FnMut(&[f32], &mut [f32]),
    {
        let n = data_set.len() as f32;
        let mut sum = 0.0;

        match self {
            Loss::MSE => {
                for sample in data_set.iter() {
                    eval_into_buf(sample.input(), buffer);
                    for (&target, &pred) in sample.output().iter().zip(buffer.iter()) {
                        let d = target - pred;
                        sum += d * d;
                    }
                }
            }
            Loss::MAE => {
                for sample in data_set.iter() {
                    eval_into_buf(sample.input(), buffer);
                    let target = sample.output();
                    for (&t, &p) in target.iter().zip(buffer.iter()) {
                        let d = t - p;
                        sum += d.abs();
                    }
                }
            }
            Loss::XEnt => {
                for sample in data_set.iter() {
                    eval_into_buf(sample.input(), buffer);
                    for (&target, &pred) in sample.output().iter().zip(buffer.iter()) {
                        let q = pred.clamp(EPS, 1.0);
                        sum += -target * q.ln();
                    }
                }
            }
            Loss::Diff => {
                for sample in data_set.iter() {
                    eval_into_buf(sample.input(), buffer);
                    let target = sample.output();
                    for (&t, &p) in target.iter().zip(buffer.iter()) {
                        sum += t - p;
                    }
                }
            }
        }

        sum / n
    }
}
