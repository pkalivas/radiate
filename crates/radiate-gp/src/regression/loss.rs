use super::DataSet;
use crate::{EvalMut, ops::GpFloat};

#[derive(Debug, Clone, Copy)]
pub enum Loss {
    MSE,
    MAE,
    XEnt,
    Diff,
}

impl Loss {
    #[inline]
    pub fn calc<F: GpFloat>(
        &self,
        data_set: &DataSet<F>,
        eval: &mut impl EvalMut<[F], Vec<F>>,
    ) -> F {
        let out_len = data_set.shape().2;
        let mut buffer = vec![F::ZERO; out_len];

        self.calculate(data_set, &mut buffer[..out_len], |x, y| {
            let v = eval.eval_mut(x);
            y.copy_from_slice(&v);
        })
    }

    #[inline]
    pub fn calculate<F, E>(
        &self,
        data_set: &DataSet<F>,
        buffer: &mut [F],
        mut eval_into_buf: E,
    ) -> F
    where
        F: GpFloat,
        E: FnMut(&[F], &mut [F]),
    {
        let n = F::from(data_set.len()).unwrap();
        let mut sum = F::ZERO;

        match self {
            Loss::MSE => {
                for sample in data_set.iter() {
                    eval_into_buf(sample.input(), buffer);
                    for (&target, &pred) in sample.output().iter().zip(buffer.iter()) {
                        let d = target - pred;
                        sum = sum + d * d;
                    }
                }
            }
            Loss::MAE => {
                for sample in data_set.iter() {
                    eval_into_buf(sample.input(), buffer);
                    let target = sample.output();
                    for (&t, &p) in target.iter().zip(buffer.iter()) {
                        let d = t - p;
                        sum = sum + d.abs();
                    }
                }
            }
            Loss::XEnt => {
                for sample in data_set.iter() {
                    eval_into_buf(sample.input(), buffer);
                    for (&target, &pred) in sample.output().iter().zip(buffer.iter()) {
                        let q = pred.clamp(F::LOG_EPS, F::ONE);
                        sum = sum - target * q.ln();
                    }
                }
            }
            Loss::Diff => {
                for sample in data_set.iter() {
                    eval_into_buf(sample.input(), buffer);
                    let target = sample.output();
                    for (&t, &p) in target.iter().zip(buffer.iter()) {
                        sum = sum + (t - p);
                    }
                }
            }
        }

        sum / n
    }
}
