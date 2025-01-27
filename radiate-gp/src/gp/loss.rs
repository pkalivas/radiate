use crate::{EvalMut, Graph, GraphEvaluator};

use super::DataSet;

const ZERO: f32 = 0.0;

#[derive(Debug, Clone, Copy)]
pub enum Loss {
    MSE,
    MAE,
    CrossEntropy,
    Diff,
}

impl Loss {
    pub fn calculate<F>(&self, samples: &DataSet, eval_func: &mut F) -> f32
    where
        F: FnMut(&Vec<f32>) -> Vec<f32>,
    {
        match self {
            Loss::MSE => {
                let mut sum = ZERO;
                for sample in samples.iter() {
                    let output = eval_func(sample.input());

                    for (i, val) in output.into_iter().enumerate() {
                        let diff = sample.output()[i] - val;
                        sum += diff * diff;
                    }
                }

                sum / (samples.iter().len() as f32)
            }
            Loss::MAE => {
                let mut sum = ZERO;
                for sample in samples.iter() {
                    let output = eval_func(sample.input());

                    for i in 0..sample.output().len() {
                        let diff = sample.output()[i] - output[i];
                        sum += diff;
                    }
                }

                sum /= samples.iter().len() as f32;
                sum
            }
            Loss::CrossEntropy => {
                let mut sum = ZERO;
                for sample in samples.iter() {
                    let output = eval_func(sample.input());

                    for i in 0..sample.output().len() {
                        sum += sample.output()[i] * output[i].ln();
                    }
                }

                sum
            }
            Loss::Diff => {
                let mut sum = ZERO;
                for sample in samples.iter() {
                    let output = eval_func(sample.input());

                    for i in 0..sample.output().len() {
                        sum += (sample.output()[i] - output[i]).abs();
                    }
                }

                sum
            }
        }
    }
}

impl EvalMut<(Graph<f32>, &DataSet), f32> for Loss {
    fn eval_mut(&mut self, (graph, samples): &(Graph<f32>, &DataSet)) -> f32 {
        let mut evaluator = GraphEvaluator::new(graph);
        self.calculate(samples, &mut |input| evaluator.eval_mut(input))
    }
}
