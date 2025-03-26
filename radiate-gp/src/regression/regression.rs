use super::{DataSet, Loss};
use crate::{Eval, EvalMut, Graph, GraphEvaluator, Op, Tree};

pub struct Regression {
    data_set: DataSet,
    loss: Loss,
}

impl Regression {
    pub fn new(sample_set: DataSet, loss: Loss) -> Self {
        Regression {
            data_set: sample_set,
            loss,
        }
    }
}

impl Eval<Graph<Op<f32>>, f32> for Regression {
    fn eval(&self, graph: &Graph<Op<f32>>) -> f32 {
        let mut evaluator = GraphEvaluator::new(graph);

        self.loss
            .calculate(&self.data_set, &mut |input| evaluator.eval_mut(input))
    }
}

impl Eval<Tree<Op<f32>>, f32> for Regression {
    fn eval(&self, tree: &Tree<Op<f32>>) -> f32 {
        self.loss
            .calculate(&self.data_set, &mut |input| vec![tree.eval(input)])
    }
}

impl Eval<Vec<Tree<Op<f32>>>, f32> for Regression {
    fn eval(&self, program: &Vec<Tree<Op<f32>>>) -> f32 {
        self.loss
            .calculate(&self.data_set, &mut |input| program.eval(input))
    }
}

// use std::{marker::PhantomData, sync::Arc};

// pub struct RegressionProblem<C, T>
// where
//     C: Chromosome,
//     T: Clone,
// {
//     data_set: DataSet,
//     loss: Loss,
//     codex: Arc<dyn Codex<C, T> + Send + Sync>,
//     _marker: PhantomData<C>,
//     _marker2: PhantomData<T>,
// }

// impl<C, T> RegressionProblem<C, T>
// where
//     C: Chromosome,
//     T: Clone,
// {
//     pub fn new<G: Codex<C, T> + Send + Sync + 'static>(
//         data_set: DataSet,
//         loss: Loss,
//         codex: G,
//     ) -> Self {
//         RegressionProblem {
//             data_set,
//             loss,
//             codex: Arc::new(codex),
//             _marker: PhantomData,
//             _marker2: PhantomData,
//         }
//     }
// }

// impl Problem<GraphChromosome<Op<f32>>, Graph<Op<f32>>>
//     for RegressionProblem<GraphChromosome<Op<f32>>, Graph<Op<f32>>>
// {
//     fn encode(&self) -> Genotype<GraphChromosome<Op<f32>>> {
//         self.codex.encode()
//     }

//     fn decode(&self, genotype: &Genotype<GraphChromosome<Op<f32>>>) -> Graph<Op<f32>> {
//         self.codex.decode(genotype)
//     }

//     fn eval(&self, individual: &Genotype<GraphChromosome<Op<f32>>>) -> Score {
//         let chrome = individual.iter().next().unwrap();
//         let mut evaluator = GraphEvaluator::new(&chrome);

//         self.loss
//             .calculate(&self.data_set, &mut |input| evaluator.eval_mut(input))
//             .into()
//     }
// }
