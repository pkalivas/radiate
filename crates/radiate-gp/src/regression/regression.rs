use super::{DataSet, Loss};
use crate::{Eval, EvalMut, Graph, GraphChromosome, GraphEvaluator, Op, Tree, TreeNode};
use radiate_core::fitness::FitnessFunction;

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
