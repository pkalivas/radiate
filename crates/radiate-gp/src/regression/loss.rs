use radiate_utils::{Float, Matrix};

#[derive(Debug, Clone, Copy)]
pub enum Loss {
    MSE,
    MAE,
    XEnt,
    Diff,
}

impl Loss {
    /// Loss over predictions already computed for every sample, e.g. by a
    /// [GraphEvaluator](crate::GraphEvaluator). `preds` is `outputs × samples`: output `j` for sample
    /// `r` is `preds[(j, r)]`. Terms are summed sample by sample, output
    /// by output; [Loss::XEnt] is the exception, scored per sample (see [Loss::xent]).
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
    pub fn calculate_columns<F: Float>(&self, labels: &Matrix<F>, preds: &Matrix<F>) -> F {
        let n = F::from(labels.rows()).unwrap();
        let mut sum = F::ZERO;

        if matches!(self, Loss::XEnt) {
            for r in 0..labels.rows() {
                let row = labels.row(r);
                let target = &row[..row.len().min(preds.rows())];
                sum = sum + Self::xent(target, |j| preds[(j, r)]);
            }
        } else {
            for r in 0..labels.rows() {
                for (j, &t) in labels.row(r).iter().take(preds.rows()).enumerate() {
                    sum = sum + self.error(t, preds[(j, r)]);
                }
            }
        }

        sum / n
    }

    /// Cross-entropy for one sample, treating the outputs as logits. `pred(j)` is output `j`
    /// for `j` in `0..target.len()`.
    ///
    /// ```text
    ///   one output:    max(z, 0) − z·t + ln(1 + e^−|z|)       sigmoid binary cross-entropy
    ///   k outputs:     −Σ t_j · (z_j − logsumexp(z))           softmax cross-entropy
    /// ```
    ///
    /// Softmax couples a sample's outputs, so unlike the other losses this can't be summed
    /// term by term.
    #[inline]
    pub(crate) fn xent<F: Float>(target: &[F], pred: impl Fn(usize) -> F) -> F {
        let k = target.len();

        if k == 1 {
            let (t, z) = (target[0], pred(0));
            return z.max(F::ZERO) - z * t + (-z.abs()).exp().ln_1p();
        }

        let mut m = F::neg_infinity();
        for j in 0..k {
            m = m.max(pred(j));
        }

        let mut s = F::ZERO;
        for j in 0..k {
            s = s + (pred(j) - m).exp();
        }

        let lse = m + s.ln();
        let mut loss = F::ZERO;
        for (j, &t) in target.iter().enumerate() {
            loss = loss - t * (pred(j) - lse);
        }

        loss
    }

    /// One sample/output's contribution to the summed loss.
    #[inline(always)]
    pub(crate) fn error<F: Float>(&self, target: F, pred: F) -> F {
        match self {
            Loss::MSE => {
                let d = target - pred;
                d * d
            }
            Loss::MAE => (target - pred).abs(),
            Loss::Diff => target - pred,
            Loss::XEnt => unreachable!("XEnt is scored per sample, see Loss::xent"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f32 = 1e-5;

    /// `preds` laid out `outputs × samples` from one row of outputs per sample.
    fn columns(samples: &[Vec<f32>]) -> Matrix<f32> {
        let outputs = samples[0].len();
        Matrix::from(
            (0..outputs)
                .map(|j| samples.iter().map(|s| s[j]).collect())
                .collect::<Vec<Vec<f32>>>(),
        )
    }

    fn xent(target: &[f32], logits: &[f32]) -> f32 {
        Loss::xent(target, |j| logits[j])
    }

    #[test]
    fn softmax_xent_matches_hand_computed() {
        // −(3 − ln(e¹ + e² + e³)) = ln(e⁻² + e⁻¹ + 1)
        let want = (1.0f32 + (-1.0f32).exp() + (-2.0f32).exp()).ln();
        assert!((xent(&[0.0, 0.0, 1.0], &[1.0, 2.0, 3.0]) - want).abs() < EPS);
    }

    #[test]
    fn binary_xent_matches_hand_computed() {
        // −ln σ(0) = ln 2, and −ln(1 − σ(2)) = 2 + ln(1 + e⁻²)
        assert!((xent(&[1.0], &[0.0]) - 2.0f32.ln()).abs() < EPS);
        let want = 2.0 + (1.0 + (-2.0f32).exp()).ln();
        assert!((xent(&[0.0], &[2.0]) - want).abs() < EPS);
    }

    #[test]
    fn large_logits_stay_finite() {
        let small = xent(&[0.0, 0.0, 1.0], &[1.0, 2.0, 3.0]);
        let large = xent(&[0.0, 0.0, 1.0], &[1000.0, 1001.0, 1002.0]);
        // f32 spacing near 1000 is ~6e-5, so compare at that resolution.
        assert!((small - large).abs() < 1e-3);
        assert!(xent(&[0.0], &[1000.0]).is_finite());
        assert!(xent(&[1.0], &[-1000.0]).is_finite());
    }

    #[test]
    fn constant_outputs_are_not_free() {
        // Equal logits are a uniform guess: ln k, not 0.
        let loss = xent(&[0.0, 1.0, 0.0], &[100.0, 100.0, 100.0]);
        assert!((loss - 3.0f32.ln()).abs() < EPS);
    }

    #[test]
    fn wrong_classes_are_penalized() {
        let right = xent(&[0.0, 1.0, 0.0], &[0.0, 5.0, 0.0]);
        let wrong = xent(&[0.0, 1.0, 0.0], &[5.0, 0.0, 0.0]);
        assert!(right < wrong);
    }

    #[test]
    fn binary_always_positive_is_penalized() {
        let labels = Matrix::from(vec![vec![1.0], vec![0.0]]);
        let always_positive = columns(&[vec![10.0], vec![10.0]]);
        let separates = columns(&[vec![10.0], vec![-10.0]]);

        let bad = Loss::XEnt.calculate_columns(&labels, &always_positive);
        let good = Loss::XEnt.calculate_columns(&labels, &separates);
        assert!(
            bad > 4.0,
            "always predicting 1 should cost ~10 on the negative, got {bad}"
        );
        assert!(good < 1e-3);
    }

    #[test]
    fn calculate_columns_averages_per_sample_xent() {
        let labels = Matrix::from(vec![vec![0.0, 0.0, 1.0], vec![1.0, 0.0, 0.0]]);
        let samples = [vec![1.0, 2.0, 3.0], vec![0.5, -1.0, 2.0]];

        let want = (xent(labels.row(0), &samples[0]) + xent(labels.row(1), &samples[1])) / 2.0;
        let got = Loss::XEnt.calculate_columns(&labels, &columns(&samples));
        assert!((got - want).abs() < EPS);
    }

    #[test]
    fn mse_is_summed_term_by_term() {
        let labels = Matrix::from(vec![vec![1.0, 2.0], vec![3.0, 4.0]]);
        let preds = columns(&[vec![0.0, 2.0], vec![3.0, 6.0]]);
        // (1 + 0 + 0 + 4) / 2 samples
        assert_eq!(Loss::MSE.calculate_columns(&labels, &preds), 2.5);
    }
}
