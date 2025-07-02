use super::{DataSet, Loss};
use crate::{Eval, EvalMut, Graph, GraphChromosome, GraphEvaluator, Op, Tree, TreeNode};
use radiate_core::problem::{FitnessFunction, Novelty};

pub struct Regression {
    data_set: DataSet,
    loss: Loss,
}

impl Regression {
    pub fn new(sample_set: impl Into<DataSet>, loss: Loss) -> Self {
        Regression {
            data_set: sample_set.into(),
            loss,
        }
    }
}

impl FitnessFunction<Graph<Op<f32>>, f32> for Regression {
    fn evaluate(&self, input: Graph<Op<f32>>) -> f32 {
        let mut evaluator = GraphEvaluator::new(&input);

        self.loss
            .calculate(&self.data_set, &mut |input| evaluator.eval_mut(input))
    }
}

impl FitnessFunction<GraphChromosome<Op<f32>>, f32> for Regression {
    fn evaluate(&self, input: GraphChromosome<Op<f32>>) -> f32 {
        let mut evaluator = GraphEvaluator::new(&input);

        self.loss
            .calculate(&self.data_set, &mut |input| evaluator.eval_mut(input))
    }
}

impl FitnessFunction<Tree<Op<f32>>, f32> for Regression {
    fn evaluate(&self, input: Tree<Op<f32>>) -> f32 {
        self.loss
            .calculate(&self.data_set, &mut |vals| vec![input.eval(vals)])
    }
}

impl FitnessFunction<Vec<Tree<Op<f32>>>, f32> for Regression {
    fn evaluate(&self, input: Vec<Tree<Op<f32>>>) -> f32 {
        self.loss.calculate(&self.data_set, &mut |vals| {
            input.iter().map(|tree| tree.eval(vals)).collect()
        })
    }
}

impl FitnessFunction<Vec<&TreeNode<Op<f32>>>, f32> for Regression {
    fn evaluate(&self, input: Vec<&TreeNode<Op<f32>>>) -> f32 {
        self.loss.calculate(&self.data_set, &mut |vals| {
            input.iter().map(|tree| tree.eval(vals)).collect()
        })
    }
}

impl Novelty<Graph<Op<f32>>> for Regression {
    type Descriptor = Vec<f32>;

    fn description(&self, phenotype: &Graph<Op<f32>>) -> Self::Descriptor {
        let mut evaluator = GraphEvaluator::new(phenotype);
        let res = self
            .data_set
            .iter()
            .map(|sample| evaluator.eval_mut(sample.input()))
            .flatten()
            .collect::<Vec<f32>>();

        res
    }

    fn distance(&self, a: &Self::Descriptor, b: &Self::Descriptor) -> f32 {
        a.iter()
            .zip(b.iter())
            .map(|(x, y)| (x - y).powi(2))
            .sum::<f32>()
            .sqrt()
    }
}
