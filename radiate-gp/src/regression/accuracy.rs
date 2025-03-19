use super::{DataSet, Loss};
use std::fmt::Debug;

#[derive(Clone)]
pub struct Accuracy<'a> {
    name: String,
    data_set: &'a DataSet,
    loss_fn: Loss,
}

impl<'a> Accuracy<'a> {
    pub fn new(name: impl Into<String>, data_set: &'a DataSet, loss_fn: Loss) -> Self {
        Accuracy {
            name: name.into(),
            data_set,
            loss_fn,
        }
    }

    pub fn calc<F>(&self, mut eval: F) -> AccuracyResult
    where
        F: FnMut(&Vec<f32>) -> Vec<f32>,
    {
        let mut outputs = Vec::new();
        let mut total_samples = 0.0;
        let mut correct_predictions = 0.0;
        let mut is_regression = true;

        let mut mae = 0.0; // Mean Absolute Error
        let mut mse = 0.0; // Mean Squared Error
        let mut min_output = f32::MAX;
        let mut max_output = f32::MIN;
        let mut ss_total = 0.0; // Sum of squares total for R²
        let mut ss_residual = 0.0; // Sum of squares residual for R²
        let mut y_mean = 0.0;

        let mut tp = 0.0; // True Positives (for classification)
        let mut fp = 0.0; // False Positives
        let mut fn_ = 0.0; // False Negatives

        let loss = self.loss_fn.calculate(self.data_set, &mut eval);

        // Compute the mean of actual values for R² calculation
        let total_values: usize = self.data_set.len();
        if total_values > 0 {
            y_mean =
                self.data_set.iter().map(|row| row.output()[0]).sum::<f32>() / total_values as f32;
        }

        for row in self.data_set.iter() {
            let output = eval(row.input());
            outputs.push(output.clone());

            if output.len() == 1 {
                // Regression case
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
                // Classification case
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
                1.0 - (mae / total_samples) / (max_output - min_output) // Scaled MAE-based accuracy
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
            0.0 // If ss_total is 0, all y_true are the same, meaning the model is perfect (or there's no variance)
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
            loss_fn: self.loss_fn,
            sample_count: self.data_set.len(),
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
