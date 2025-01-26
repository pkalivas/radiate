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

impl Problem<GraphChromosome<f32>, Graph<f32>>
    for RegressionProblem<GraphChromosome<f32>, Graph<f32>>
{
    fn encode(&self) -> Genotype<GraphChromosome<f32>> {
        self.codex.encode()
    }

    fn decode(&self, genotype: &Genotype<GraphChromosome<f32>>) -> Graph<f32> {
        self.codex.decode(genotype)
    }

    fn eval(&self, graph: &Genotype<GraphChromosome<f32>>) -> Score {
        let graph = &graph.chromosomes[0].as_ref();
        let mut evaluator = GraphEvaluator::new(graph);

        self.loss_function
            .calculate(&self.data_set, &mut |input| evaluator.eval_mut(input))
            .into()
    }
}
