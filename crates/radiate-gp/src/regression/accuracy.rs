use super::{DataSet, Loss};
use crate::EvalMut;
use std::fmt::Debug;

#[derive(Clone)]
pub struct Accuracy<'a> {
    name: String,
    data_set: Option<&'a DataSet>,
    loss_fn: Option<Loss>,
}

impl<'a> Accuracy<'a> {
    pub fn new(name: impl Into<String>) -> Self {
        Accuracy {
            name: name.into(),
            data_set: None,
            loss_fn: None,
        }
    }

    pub fn on(mut self, data_set: &'a DataSet) -> Self {
        self.data_set = Some(data_set);
        self
    }

    pub fn loss(mut self, loss_fn: Loss) -> Self {
        self.loss_fn = Some(loss_fn);
        self
    }

    pub fn calc(&self, eval: &mut impl EvalMut<[f32], Vec<f32>>) -> AccuracyResult {
        let data_set = self
            .data_set
            .expect("DataSet reference must be provided for accuracy calculation");
        let loss_fn = self
            .loss_fn
            .expect("Loss function must be provided for accuracy calculation");

        self.calc_internal(eval, data_set, loss_fn)
    }

    pub fn calc_internal(
        &self,
        eval: &mut impl EvalMut<[f32], Vec<f32>>,
        data_set: &DataSet,
        loss_fn: Loss,
    ) -> AccuracyResult {
        let mut outputs = Vec::new();
        let mut total_samples = 0.0;
        let mut correct_predictions = 0.0;
        let mut is_regression = true;

        let mut mae = 0.0;
        let mut mse = 0.0;
        let mut min_output = f32::MAX;
        let mut max_output = f32::MIN;
        let mut ss_total = 0.0;
        let mut ss_residual = 0.0;
        let mut y_mean = 0.0;

        let mut tp = 0.0;
        let mut fp = 0.0;
        let mut fn_ = 0.0;

        let loss = loss_fn.calc(data_set, eval);

        let total_values = data_set.len();
        if total_values > 0 {
            y_mean = data_set.iter().map(|row| row.output()[0]).sum::<f32>() / total_values as f32;
        }

        for row in data_set.iter() {
            let output = eval.eval_mut(row.input());
            outputs.push(output.clone());

            if output.len() == 1 {
                is_regression = true;
                let y_true = row.output()[0];
                let y_pred = output[0];

                mae += (y_true - y_pred).abs();
                mse += (y_true - y_pred).powi(2);
                ss_residual += (y_true - y_pred).powi(2);
                ss_total += (y_true - y_mean).powi(2);

                min_output = min_output.min(y_true);
                max_output = max_output.max(y_true);
                total_samples += 1.0;
            } else {
                is_regression = false;
                if let Some((max_idx, _)) = output
                    .iter()
                    .enumerate()
                    .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
                {
                    if let Some(target) = row.output().iter().position(|&x| x == 1.0) {
                        total_samples += 1.0;
                        if max_idx == target {
                            correct_predictions += 1.0;
                            tp += 1.0;
                        } else {
                            fp += 1.0;
                        }
                    } else {
                        fn_ += 1.0;
                    }
                }
            }
        }

        // Compute final accuracy
        let accuracy = if is_regression {
            if total_samples > 0.0 && (max_output - min_output) > 0.0 {
                1.0 - (mae / total_samples) / (max_output - min_output)
            } else {
                0.0
            }
        } else if total_samples > 0.0 {
            correct_predictions / total_samples
        } else {
            0.0
        };

        // Compute classification metrics only if it's a classification task
        let (precision, recall, f1_score) = if is_regression {
            (0.0, 0.0, 0.0) // Not applicable for regression
        } else {
            let precision = if tp + fp > 0.0 { tp / (tp + fp) } else { 0.0 };
            let recall = if tp + fn_ > 0.0 { tp / (tp + fn_) } else { 0.0 };
            let f1_score = if precision + recall > 0.0 {
                2.0 * (precision * recall) / (precision + recall)
            } else {
                0.0
            };
            (precision, recall, f1_score)
        };

        let rmse = if total_samples > 0.0 {
            (mse / total_samples).sqrt()
        } else {
            0.0
        };

        // Compute R² score
        let r_squared = if ss_total > 0.0 {
            1.0 - (ss_residual / ss_total)
        } else {
            0.0
        };

        AccuracyResult {
            name: self.name.clone(),
            accuracy,
            precision,
            recall,
            f1_score,
            rmse,
            r_squared,
            loss,
            loss_fn,
            sample_count: data_set.len(),
            is_regression,
        }
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

impl Debug for AccuracyResult {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.is_regression {
            write!(
                f,
                "Regression Accuracy - {:?} {{\n\tN: {:?} \n\tAccuracy: {:.2}%\n\tR² Score: {:.5}\n\tRMSE: {:.5}\n\tLoss ({:?}): {:.5}\n}}",
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
                "Classification Accuracy - {:?} {{\n\tN: {:?} \n\tAccuracy: {:.2}%\n\tPrecision: {:.2}%\n\tRecall: {:.2}%\n\tF1 Score: {:.2}%\n\tLoss ({:?}): {:.5}\n}}",
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
