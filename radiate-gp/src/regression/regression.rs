use super::{DataSet, Loss};
use crate::{Eval, EvalMut, Graph, GraphEvaluator, Tree};

pub struct Regression {
    data_set: DataSet,
    loss_function: Loss,
}

impl Regression {
    pub fn new(sample_set: DataSet, loss_function: Loss) -> Self {
        Regression {
            data_set: sample_set,
            loss_function,
        }
    }
}

impl Eval<Graph<f32>, f32> for Regression {
    fn eval(&self, graph: &Graph<f32>) -> f32 {
        let mut evaluator = GraphEvaluator::new(graph);

        self.loss_function
            .calculate(&self.data_set, &mut |input| evaluator.eval_mut(input))
    }
}

impl Eval<Tree<f32>, f32> for Regression {
    fn eval(&self, tree: &Tree<f32>) -> f32 {
        self.loss_function
            .calculate(&self.data_set, &mut |input| vec![tree.eval(input)])
    }
}
