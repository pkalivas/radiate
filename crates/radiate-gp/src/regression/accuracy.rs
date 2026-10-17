use super::Loss;
use crate::{Eval, EvalMut, Graph, GraphEvaluator, Op, Tree};
use radiate_utils::{DataSet, Float};
use std::fmt::Debug;

#[derive(Clone, Default)]
pub struct Accuracy<'a, F: Float> {
    name: Option<String>,
    data_set: Option<&'a DataSet<F>>,
    loss_fn: Option<Loss>,
}

impl<'a, F: Float> Accuracy<'a, F> {
    pub fn named(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn on(mut self, data_set: &'a DataSet<F>) -> Self {
        self.data_set = Some(data_set);
        self
    }

    pub fn loss(mut self, loss_fn: Loss) -> Self {
        self.loss_fn = Some(loss_fn);
        self
    }

    pub fn calc(&self, eval: &mut impl EvalMut<[F], Vec<F>>) -> AccuracyResult {
        let data_set = self
            .data_set
            .expect("DataSet reference must be provided for accuracy calculation");
        let loss_fn = self
            .loss_fn
            .expect("Loss function must be provided for accuracy calculation");

        self.calc_internal(eval, data_set, loss_fn)
    }

    /// Evaluates every row of `data_set` exactly once and derives every metric from that
    /// single pass, so stateful evaluators (e.g. recurrent graphs) report loss and accuracy
    /// for the same predictions.
    ///
    /// A single label column is scored as regression. More than one is scored as one-hot
    /// classification: the argmax of the outputs against the hot label.
    pub fn calc_internal(
        &self,
        eval: &mut impl EvalMut<[F], Vec<F>>,
        data_set: &DataSet<F>,
        loss_fn: Loss,
    ) -> AccuracyResult {
        let (sample_count, _, output_dim) = data_set.shape();
        let is_regression = output_dim <= 1;

        let mut loss_sum = F::ZERO;
        let mut regression = RegressionStats::new(data_set);
        let mut classification = ClassificationStats::new(output_dim);

        for (input, target) in data_set.iter() {
            let output = eval.eval_mut(input);

            for (&t, &p) in target.iter().zip(output.iter()) {
                loss_sum = loss_sum + loss_fn.error(t, p);
            }

            if is_regression {
                if let (Some(&y_true), Some(&y_pred)) = (target.first(), output.first()) {
                    regression.push(y_true, y_pred);
                }
            } else {
                classification.push(target, &output);
            }
        }

        let loss = if sample_count > 0 {
            loss_sum / F::from(sample_count).unwrap()
        } else {
            F::ZERO
        };

        let (accuracy, precision, recall, f1_score, rmse, r_squared) = if is_regression {
            let (accuracy, rmse, r_squared) = regression.finish();
            (accuracy, 0.0, 0.0, 0.0, rmse, r_squared)
        } else {
            let (accuracy, precision, recall, f1_score) = classification.finish();
            (accuracy, precision, recall, f1_score, 0.0, 0.0)
        };

        AccuracyResult {
            name: match &self.name {
                Some(name) => name.clone(),
                None => {
                    if is_regression {
                        "Regression Accuracy".to_string()
                    } else {
                        "Classification Accuracy".to_string()
                    }
                }
            },
            accuracy,
            precision,
            recall,
            f1_score,
            rmse,
            r_squared,
            loss: loss.extract().unwrap_or(0.0),
            loss_fn,
            sample_count,
            is_regression,
        }
    }
}

/// Running sums for single-output regression. The label mean and range come from a
/// labels-only pre-pass (no evaluation) so `R²` can be accumulated in the main pass.
struct RegressionStats<F: Float> {
    samples: F,
    abs_error: F,
    sq_error: F,
    ss_total: F,
    y_mean: F,
    y_min: F,
    y_max: F,
}

impl<F: Float> RegressionStats<F> {
    fn new(data_set: &DataSet<F>) -> Self {
        let mut sum = F::ZERO;
        let mut y_min = F::MAX;
        let mut y_max = F::MIN;

        for (_, target) in data_set.iter() {
            if let Some(&y) = target.first() {
                sum = sum + y;
                y_min = y_min.min(y);
                y_max = y_max.max(y);
            }
        }

        let y_mean = if data_set.is_empty() {
            F::ZERO
        } else {
            sum / F::from(data_set.len()).unwrap()
        };

        RegressionStats {
            samples: F::ZERO,
            abs_error: F::ZERO,
            sq_error: F::ZERO,
            ss_total: F::ZERO,
            y_mean,
            y_min,
            y_max,
        }
    }

    fn push(&mut self, y_true: F, y_pred: F) {
        let residual = y_true - y_pred;
        self.abs_error = self.abs_error + residual.abs();
        self.sq_error = self.sq_error + residual * residual;
        self.ss_total = self.ss_total + (y_true - self.y_mean).powi(2);
        self.samples = self.samples + F::ONE;
    }

    /// Returns `(accuracy, rmse, r_squared)`. Accuracy here is `1 - MAE / range(y)`, a
    /// normalized error rather than a standard metric.
    fn finish(&self) -> (f32, f32, f32) {
        let range = self.y_max - self.y_min;

        let accuracy = if self.samples > F::ZERO && range > F::ZERO {
            F::ONE - (self.abs_error / self.samples) / range
        } else {
            F::ZERO
        };

        let rmse = if self.samples > F::ZERO {
            (self.sq_error / self.samples).sqrt()
        } else {
            F::ZERO
        };

        let r_squared = if self.ss_total > F::ZERO {
            F::ONE - (self.sq_error / self.ss_total)
        } else {
            F::ZERO
        };

        (
            accuracy.extract().unwrap_or(0.0),
            rmse.extract().unwrap_or(0.0),
            r_squared.extract().unwrap_or(0.0),
        )
    }
}

/// Per-class confusion counts for one-hot classification. Precision, recall and F1 are
/// macro-averaged over every class that appears in the labels or the predictions.
struct ClassificationStats {
    true_pos: Vec<usize>,
    false_pos: Vec<usize>,
    false_neg: Vec<usize>,
    correct: usize,
    total: usize,
}

impl ClassificationStats {
    fn new(classes: usize) -> Self {
        ClassificationStats {
            true_pos: vec![0; classes],
            false_pos: vec![0; classes],
            false_neg: vec![0; classes],
            correct: 0,
            total: 0,
        }
    }

    /// Rows without a hot label are skipped. A prediction that contains `NaN` (or points
    /// past the label width) is counted as a miss for the true class.
    fn push<F: Float>(&mut self, target: &[F], output: &[F]) {
        let Some(label) = target.iter().position(|&x| x == F::ONE) else {
            return;
        };

        self.total += 1;

        let predicted = if output.iter().any(|v| v.is_nan()) {
            None
        } else {
            output
                .iter()
                .enumerate()
                .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
                .map(|(idx, _)| idx)
        };

        match predicted {
            Some(idx) if idx == label => {
                self.true_pos[label] += 1;
                self.correct += 1;
            }
            Some(idx) => {
                if let Some(fp) = self.false_pos.get_mut(idx) {
                    *fp += 1;
                }
                self.false_neg[label] += 1;
            }
            None => self.false_neg[label] += 1,
        }
    }

    /// Returns `(accuracy, precision, recall, f1_score)`.
    fn finish(&self) -> (f32, f32, f32, f32) {
        let ratio = |num: usize, den: usize| {
            if den > 0 {
                num as f32 / den as f32
            } else {
                0.0
            }
        };

        let mut classes = 0;
        let (mut precision, mut recall, mut f1_score) = (0.0, 0.0, 0.0);

        for c in 0..self.true_pos.len() {
            let (tp, fp, fn_) = (self.true_pos[c], self.false_pos[c], self.false_neg[c]);
            if tp + fp + fn_ == 0 {
                continue;
            }

            let p = ratio(tp, tp + fp);
            let r = ratio(tp, tp + fn_);

            precision += p;
            recall += r;
            f1_score += if p + r > 0.0 {
                2.0 * p * r / (p + r)
            } else {
                0.0
            };
            classes += 1;
        }

        let classes = classes.max(1) as f32;

        (
            ratio(self.correct, self.total),
            precision / classes,
            recall / classes,
            f1_score / classes,
        )
    }
}

pub struct AccuracyResult {
    name: String,
    accuracy: f32,
    precision: f32, // Only for classification
    recall: f32,    // Only for classification
    f1_score: f32,  // Only for classification
    rmse: f32,      // Only for regression
    r_squared: f32, // Only for regression
    sample_count: usize,
    loss: f32,
    loss_fn: Loss,
    is_regression: bool,
}

impl AccuracyResult {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn accuracy(&self) -> f32 {
        self.accuracy
    }

    pub fn precision(&self) -> f32 {
        self.precision
    }

    pub fn recall(&self) -> f32 {
        self.recall
    }

    pub fn f1_score(&self) -> f32 {
        self.f1_score
    }

    pub fn rmse(&self) -> f32 {
        self.rmse
    }

    pub fn r_squared(&self) -> f32 {
        self.r_squared
    }

    pub fn sample_count(&self) -> usize {
        self.sample_count
    }

    pub fn loss(&self) -> f32 {
        self.loss
    }

    pub fn loss_fn(&self) -> Loss {
        self.loss_fn
    }
}

impl Debug for AccuracyResult {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.is_regression {
            write!(
                f,
                "{:?} {{\n\tN: {:?} \n\tAccuracy: {:.2}%\n\tR² Score: {:.5}\n\tRMSE: {:.5}\n\tLoss ({:?}): {:.5}\n}}",
                self.name,
                self.sample_count,
                self.accuracy * 100.0,
                self.r_squared,
                self.rmse,
                self.loss_fn,
                self.loss
            )
        } else {
            write!(
                f,
                "{:?} {{\n\tN: {:?} \n\tAccuracy: {:.2}%\n\tPrecision: {:.2}%\n\tRecall: {:.2}%\n\tF1 Score: {:.2}%\n\tLoss ({:?}): {:.5}\n}}",
                self.name,
                self.sample_count,
                self.accuracy * 100.0,
                self.precision * 100.0,
                self.recall * 100.0,
                self.f1_score * 100.0,
                self.loss_fn,
                self.loss
            )
        }
    }
}

impl<T: Float> Eval<Graph<Op<T>>, Option<AccuracyResult>> for Accuracy<'_, T> {
    fn eval(&self, graph: &Graph<Op<T>>) -> Option<AccuracyResult> {
        let mut evaluator = GraphEvaluator::new(graph);
        Some(self.calc(&mut evaluator))
    }
}

impl<T: Float> Eval<Tree<Op<T>>, Option<AccuracyResult>> for Accuracy<'_, T> {
    fn eval(&self, tree: &Tree<Op<T>>) -> Option<AccuracyResult> {
        Some(self.calc(&mut tree.clone()))
    }
}

impl<T: Float> Eval<Vec<Tree<Op<T>>>, Option<AccuracyResult>> for Accuracy<'_, T> {
    fn eval(&self, trees: &Vec<Tree<Op<T>>>) -> Option<AccuracyResult> {
        let mut cloned_trees = trees.clone();
        Some(self.calc(&mut cloned_trees))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Replays fixed predictions in order and counts how many times it was evaluated.
    struct Replay {
        preds: Vec<Vec<f32>>,
        calls: usize,
    }

    impl EvalMut<[f32], Vec<f32>> for Replay {
        fn eval_mut(&mut self, _: &[f32]) -> Vec<f32> {
            let out = self.preds[self.calls % self.preds.len()].clone();
            self.calls += 1;
            out
        }
    }

    fn one_hot(class: usize, classes: usize) -> Vec<f32> {
        (0..classes)
            .map(|c| if c == class { 1.0 } else { 0.0 })
            .collect()
    }

    #[test]
    fn evaluates_each_row_once() {
        let data = DataSet::new(vec![vec![0.0]; 5], vec![vec![1.0]; 5]);
        let mut eval = Replay {
            preds: vec![vec![1.0]],
            calls: 0,
        };

        Accuracy::default()
            .on(&data)
            .loss(Loss::MSE)
            .calc(&mut eval);

        assert_eq!(eval.calls, 5);
    }

    #[test]
    fn perfect_regression() {
        let labels = vec![vec![1.0], vec![2.0], vec![4.0]];
        let data = DataSet::new(vec![vec![0.0]; 3], labels.clone());
        let mut eval = Replay {
            preds: labels,
            calls: 0,
        };

        let result = Accuracy::default()
            .on(&data)
            .loss(Loss::MSE)
            .calc(&mut eval);

        assert_eq!(result.accuracy(), 1.0);
        assert_eq!(result.rmse(), 0.0);
        assert_eq!(result.r_squared(), 1.0);
        assert_eq!(result.loss(), 0.0);
    }

    #[test]
    fn classification_macro_metrics() {
        // labels: 0, 0, 1, 2 -- predictions: 0, 1, 1, 1
        //   class 0: tp 1, fp 0, fn 1 -> p 1,   r 1/2, f1 2/3
        //   class 1: tp 1, fp 2, fn 0 -> p 1/3, r 1,   f1 1/2
        //   class 2: tp 0, fp 0, fn 1 -> p 0,   r 0,   f1 0
        let labels = [0, 0, 1, 2].map(|c| one_hot(c, 3)).to_vec();
        let preds = [0, 1, 1, 1].map(|c| one_hot(c, 3)).to_vec();
        let data = DataSet::new(vec![vec![0.0]; 4], labels);
        let mut eval = Replay { preds, calls: 0 };

        let result = Accuracy::default()
            .on(&data)
            .loss(Loss::MSE)
            .calc(&mut eval);

        let close = |a: f32, b: f32| (a - b).abs() < 1e-6;
        assert!(close(result.accuracy(), 0.5));
        assert!(close(result.precision(), (1.0 + 1.0 / 3.0) / 3.0));
        assert!(close(result.recall(), 1.5 / 3.0));
        assert!(close(result.f1_score(), (2.0 / 3.0 + 0.5) / 3.0));
    }

    #[test]
    fn nan_prediction_counts_as_miss() {
        let labels = vec![one_hot(0, 2), one_hot(1, 2)];
        let preds = vec![vec![f32::NAN, 0.0], one_hot(1, 2)];
        let data = DataSet::new(vec![vec![0.0]; 2], labels);
        let mut eval = Replay { preds, calls: 0 };

        let result = Accuracy::default()
            .on(&data)
            .loss(Loss::MSE)
            .calc(&mut eval);

        assert_eq!(result.accuracy(), 0.5);
    }
}
