use std::fmt::Debug;

use super::{DataSet, Loss};

pub struct Accuracy<'a> {
    name: &'a str,
    data_set: &'a DataSet,
    loss_fn: Loss,
}

impl<'a> Accuracy<'a> {
    pub fn new(name: &'a str, data_set: &'a DataSet, loss_fn: Loss) -> Self {
        Accuracy {
            name,
            data_set,
            loss_fn,
        }
    }

    pub fn calc<F>(&self, mut eval: F) -> AccuracyResult
    where
        F: FnMut(&Vec<f32>) -> Vec<f32>,
    {
        let mut outputs = Vec::new();
        let mut single_accuracy = true;
        let mut accuracy = 0.0;
        let mut total = 0.0;

        let loss = self.loss_fn.calculate(&self.data_set, &mut eval);

        for row in self.data_set.iter() {
            let output = eval(row.input());

            match output.len() {
                1 => {
                    total += row.output()[0].abs();
                    accuracy += (row.output()[0] - output[0]).abs();
                }
                _ => {
                    single_accuracy = false;
                    let mut max_idx = 0;
                    for i in 0..output.len() {
                        if output[i] > output[max_idx] {
                            max_idx = i;
                        }
                    }

                    let target = row.output().iter().position(|&x| x == 1.0).unwrap();

                    total += 1.0;
                    if max_idx == target {
                        accuracy += 1.0;
                    }
                }
            }

            outputs.push(output);
        }

        if single_accuracy {
            accuracy = (total - accuracy) / total;
        } else {
            accuracy /= total;
        }

        AccuracyResult {
            name: self.name.to_string(),
            accuracy,
            outputs,
            loss,
            loss_fn: self.loss_fn.clone(),
            sample_count: self.data_set.len(),
        }
    }
}

pub struct AccuracyResult {
    name: String,
    accuracy: f32,
    sample_count: usize,
    loss: f32,
    // precision: f32,
    // l2: f32,
    // recall: f32,
    loss_fn: Loss,
    outputs: Vec<Vec<f32>>,
}

impl Debug for AccuracyResult {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.outputs.len() > 0 {
            if self.outputs[0].len() == 1 {
                write!(
                    f,
                    "Regression Accuracy - {:?} {{\n\tN: {:?} \n\tAccuracy: {:.2}%\n\tLoss ({:?}): {:.5}\n}}",
                    self.name,
                    self.sample_count,
                    self.accuracy * 100.0,
                    self.loss_fn,
                    self.loss
                )
            } else {
                write!(
                    f,
                    "Classification Accuracy - {:?} {{\n\tN: {:?} \n\tAccuracy: {:.2}%\n\tLoss ({:?}): {:.5}\n}}",
                    self.name,
                    self.sample_count,
                    self.accuracy * 100.0,
                    self.loss_fn,
                    self.loss
                )
            }
        } else {
            write!(f, "Accuracy: {:.2}", self.accuracy)
        }
    }
}
