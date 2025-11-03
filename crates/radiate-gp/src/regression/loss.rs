use super::DataSet;
use crate::EvalMut;

#[derive(Debug, Clone, Copy)]
pub enum Loss {
    MSE,
    MAE,
    CrossEntropy,
    Diff,
}

impl Loss {
    #[inline]
    pub fn calc(&self, data_set: &DataSet, eval: &mut impl EvalMut<[f32], Vec<f32>>) -> f32 {
        let out_len = data_set.shape().2;
        let mut buffer = vec![0.0; out_len];

        self.calculate(
            data_set,
            |x, y| {
                let v = eval.eval_mut(x);
                y.copy_from_slice(&v);
            },
            &mut buffer[..out_len],
        )
    }

    #[inline]
    pub fn calculate<F>(&self, data_set: &DataSet, mut eval_into_buf: F, buffer: &mut [f32]) -> f32
    where
        F: FnMut(&[f32], &mut [f32]),
    {
        let n = data_set.len() as f32;

        match self {
            Loss::MSE => {
                let mut sum = 0.0;
                for sample in data_set.iter() {
                    eval_into_buf(sample.input(), buffer);
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
                    eval_into_buf(sample.input(), buffer);
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
                    eval_into_buf(sample.input(), buffer);
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
                    eval_into_buf(sample.input(), buffer);
                    let target = sample.output();
                    for i in 0..target.len() {
                        sum += (target[i] - buffer[i]).abs();
                    }
                }
                sum / n
            }
        }
    }
}
