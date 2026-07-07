use super::{DataSet, Loss};
use crate::{Eval, EvalMut, Graph, GraphEvaluator, Op, Tree, ops::GpFloat};
use std::fmt::Debug;

#[derive(Clone, Default)]
pub struct Accuracy<'a, F: GpFloat> {
    name: Option<String>,
    data_set: Option<&'a DataSet<F>>,
    loss_fn: Option<Loss>,
}

impl<'a, F: GpFloat> Accuracy<'a, F> {
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

    pub fn calc_internal(
        &self,
        eval: &mut impl EvalMut<[F], Vec<F>>,
        data_set: &DataSet<F>,
        loss_fn: Loss,
    ) -> AccuracyResult {
        let mut outputs = Vec::new();
        let mut total_samples = F::ZERO;
        let mut correct_predictions = F::ZERO;
        let mut is_regression = true;

        let mut mae = F::ZERO;
        let mut mse = F::ZERO;
        let mut min_output = F::MAX;
        let mut max_output = F::MIN;
        let mut ss_total = F::ZERO;
        let mut ss_residual = F::ZERO;
        let mut y_mean = F::ZERO;

        let mut tp = F::ZERO;
        let mut fp = F::ZERO;
        let mut fn_ = F::ZERO;

        let loss = loss_fn.calc(data_set, eval);

        let total_values = data_set.len();
        if total_values > 0 {
            let mut sum = F::ZERO;
            for row in data_set.iter() {
                sum = sum + row.output()[0];
            }
            y_mean = sum / F::from(total_values).unwrap();
        }

        for row in data_set.iter() {
            let output = eval.eval_mut(row.input());
            outputs.push(output.clone());

            if output.len() == 1 {
                is_regression = true;
                let y_true = row.output()[0];
                let y_pred = output[0];

                mae = mae + (y_true - y_pred).abs();
                mse = mse + (y_true - y_pred).powi(2);
                ss_residual = ss_residual + (y_true - y_pred).powi(2);
                ss_total = ss_total + (y_true - y_mean).powi(2);

                min_output = min_output.min(y_true);
                max_output = max_output.max(y_true);
                total_samples = total_samples + F::ONE;
            } else {
                is_regression = false;
                if let Some((max_idx, _)) = output
                    .iter()
                    .enumerate()
                    .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
                {
                    if let Some(target) = row.output().iter().position(|&x| x == F::ONE) {
                        total_samples = total_samples + F::ONE;
                        if max_idx == target {
                            correct_predictions = correct_predictions + F::ONE;
                            tp = tp + F::ONE;
                        } else {
                            fp = fp + F::ONE;
                        }
                    } else {
                        fn_ = fn_ + F::ONE;
                    }
                }
            }
        }

        // Compute final accuracy
        let accuracy = if is_regression {
            if total_samples > F::ZERO && (max_output - min_output) > F::ZERO {
                F::ONE - (mae / total_samples) / (max_output - min_output)
            } else {
                F::ZERO
            }
        } else if total_samples > F::ZERO {
            correct_predictions / total_samples
        } else {
            F::ZERO
        };

        // Compute classification metrics only if it's a classification task
        let (precision, recall, f1_score) = if is_regression {
            (F::ZERO, F::ZERO, F::ZERO) // Not applicable for regression
        } else {
            let precision = if tp + fp > F::ZERO {
                tp / (tp + fp)
            } else {
                F::ZERO
            };
            let recall = if tp + fn_ > F::ZERO {
                tp / (tp + fn_)
            } else {
                F::ZERO
            };
            let f1_score = if precision + recall > F::ZERO {
                F::TWO * (precision * recall) / (precision + recall)
            } else {
                F::ZERO
            };
            (precision, recall, f1_score)
        };

        let rmse = if total_samples > F::ZERO {
            (mse / total_samples).sqrt()
        } else {
            F::ZERO
        };

        // Compute R² score
        let r_squared = if ss_total > F::ZERO {
            F::ONE - (ss_residual / ss_total)
        } else {
            F::ZERO
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
            accuracy: accuracy.extract().unwrap_or(0.0),
            precision: precision.extract().unwrap_or(0.0),
            recall: recall.extract().unwrap_or(0.0),
            f1_score: f1_score.extract().unwrap_or(0.0),
            rmse: rmse.extract().unwrap_or(0.0),
            r_squared: r_squared.extract().unwrap_or(0.0),
            loss: loss.extract().unwrap_or(0.0),
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

impl<T: GpFloat> Eval<Graph<Op<T>>, Option<AccuracyResult>> for Accuracy<'_, T> {
    fn eval(&self, graph: &Graph<Op<T>>) -> Option<AccuracyResult> {
        let mut evaluator = GraphEvaluator::new(graph);
        Some(self.calc(&mut evaluator))
    }
}

impl<T: GpFloat> Eval<Tree<Op<T>>, Option<AccuracyResult>> for Accuracy<'_, T> {
    fn eval(&self, tree: &Tree<Op<T>>) -> Option<AccuracyResult> {
        Some(self.calc(&mut tree.clone()))
    }
}

impl<T: GpFloat> Eval<Vec<Tree<Op<T>>>, Option<AccuracyResult>> for Accuracy<'_, T> {
    fn eval(&self, trees: &Vec<Tree<Op<T>>>) -> Option<AccuracyResult> {
        let mut cloned_trees = trees.clone();
        Some(self.calc(&mut cloned_trees))
    }
}
