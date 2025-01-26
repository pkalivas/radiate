use std::sync::Arc;

use radiate::{Chromosome, Codex, Genotype, Problem, Score};

use crate::{Eval, EvalMut, Graph, GraphChromosome, GraphEvaluator, Op, Tree};

use super::{DataSet, Loss};

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

impl Eval<Graph<Op<f32>>, f32> for Regression {
    fn eval(&self, graph: &Graph<Op<f32>>) -> f32 {
        let mut evaluator = GraphEvaluator::new(graph);

        self.loss_function
            .calculate(&self.data_set, &mut |input| evaluator.eval_mut(input))
    }
}

impl Eval<Tree<Op<f32>>, f32> for Regression {
    fn eval(&self, tree: &Tree<Op<f32>>) -> f32 {
        self.loss_function
            .calculate(&self.data_set, &mut |input| vec![tree.eval(input)])
    }
}

pub struct RegressionProblem<C: Chromosome, T> {
    data_set: DataSet,
    loss_function: Loss,
    codex: Arc<dyn Codex<C, T> + Send + Sync>,
}

impl<C: Chromosome, T> RegressionProblem<C, T> {
    pub fn new(
        data_set: DataSet,
        loss_function: Loss,
        codex: Arc<dyn Codex<C, T> + Send + Sync>,
    ) -> Self {
        RegressionProblem {
            data_set,
            loss_function,
            codex,
        }
    }
}

impl Problem<GraphChromosome<Op<f32>>, Graph<Op<f32>>>
    for RegressionProblem<GraphChromosome<Op<f32>>, Graph<Op<f32>>>
{
    fn encode(&self) -> Genotype<GraphChromosome<Op<f32>>> {
        self.codex.encode()
    }

    fn decode(&self, genotype: &Genotype<GraphChromosome<Op<f32>>>) -> Graph<Op<f32>> {
        self.codex.decode(genotype)
    }

    fn eval(&self, graph: &Genotype<GraphChromosome<Op<f32>>>) -> Score {
        let graph = &graph.chromosomes[0].as_ref();
        let mut evaluator = GraphEvaluator::new(graph);

        self.loss_function
            .calculate(&self.data_set, &mut |input| evaluator.eval_mut(input))
            .into()
    }
}
