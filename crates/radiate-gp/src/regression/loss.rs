use super::DataSet;
use crate::{EvalMut, ops::OpFloat};
use radiate_utils::Matrix;

#[derive(Debug, Clone, Copy)]
pub enum Loss {
    MSE,
    MAE,
    XEnt,
    Diff,
}

impl Loss {
    #[inline]
    pub fn calc<F: OpFloat>(
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
        F: OpFloat,
        E: FnMut(&[F], &mut [F]),
    {
        let n = F::from(data_set.len()).unwrap();
        let mut sum = F::ZERO;

        for (input, target) in data_set.iter() {
            eval_into_buf(input, buffer);
            for (&t, &p) in target.iter().zip(buffer.iter()) {
                sum = sum + self.error(t, p);
            }
        }

        sum / n
    }

    /// Loss over predictions already computed for every sample, e.g. by a
    /// [GraphKernel](crate::GraphKernel). `preds` is `outputs × samples`: output `j` for sample
    /// `r` is `preds[(j, r)]`. Terms are summed in the same order as [Loss::calculate] (sample by
    /// sample, output by output), so the result is bit-identical.
    ///
    /// ```text
    ///   labels (samples × outputs)        preds (outputs × samples)
    ///          j=0  j=1  j=2                       sample →  0     1    ...
    ///   r=0  [ 1    0    0  ]             j=0     [ 0.91  0.08  ...  ]
    ///   r=1  [ 0    1    0  ]             j=1     [ 0.05  0.88  ...  ]
    ///   ...                               j=2     [ 0.02  0.03  ...  ]
    ///
    ///   for r in samples:  for j in outputs:  sum += term(labels[(r, j)], preds[(j, r)])
    /// ```
    #[inline]
    pub fn calculate_columns<F: OpFloat>(&self, labels: &Matrix<F>, preds: &Matrix<F>) -> F {
        let n = F::from(labels.rows()).unwrap();
        let mut sum = F::ZERO;

        for r in 0..labels.rows() {
            for (j, &t) in labels.row(r).iter().take(preds.rows()).enumerate() {
                sum = sum + self.error(t, preds[(j, r)]);
            }
        }

        sum / n
    }

    /// One sample/output's contribution to the summed loss.
    #[inline(always)]
    fn error<F: OpFloat>(&self, target: F, pred: F) -> F {
        match self {
            Loss::MSE => {
                let d = target - pred;
                d * d
            }
            Loss::MAE => (target - pred).abs(),
            Loss::XEnt => -(target * pred.clamp(F::LOG_EPS, F::ONE).ln()),
            Loss::Diff => target - pred,
        }
    }
}
