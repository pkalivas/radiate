use super::{DataSet, Loss};
use crate::{
    Eval, EvalMut, Graph, GraphChromosome, GraphEvaluator, Op, Tree, TreeChromosome, TreeNode,
};
use radiate_core::{Chromosome, Codex, Genotype, Problem, Score};
use std::{marker::PhantomData, sync::Arc};

pub struct Regression<C, T>
where
    C: Chromosome,
    T: Clone,
{
    data_set: DataSet,
    loss: Loss,
    codex: Arc<dyn Codex<C, T>>,
    _chrom: PhantomData<C>,
    _val: PhantomData<T>,
}

impl<C, T> Regression<C, T>
where
    C: Chromosome,
    T: Clone,
{
    pub fn new<G: Codex<C, T> + 'static>(
        sample_set: impl Into<DataSet>,
        loss: Loss,
        codex: G,
    ) -> Self {
        Regression {
            data_set: sample_set.into(),
            loss,
            codex: Arc::new(codex),
            _chrom: PhantomData,
            _val: PhantomData,
        }
    }
}

impl Problem<GraphChromosome<Op<f32>>, Graph<Op<f32>>>
    for Regression<GraphChromosome<Op<f32>>, Graph<Op<f32>>>
{
    fn encode(&self) -> Genotype<GraphChromosome<Op<f32>>> {
        self.codex.encode()
    }

    fn decode(&self, genotype: &Genotype<GraphChromosome<Op<f32>>>) -> Graph<Op<f32>> {
        self.codex.decode(genotype)
    }

    fn eval(&self, individual: &Genotype<GraphChromosome<Op<f32>>>) -> Score {
        let chrome = individual.iter().next().unwrap();
        let mut evaluator = GraphEvaluator::new(chrome);

        self.loss
            .calculate(&self.data_set, &mut |input| evaluator.eval_mut(input))
            .into()
    }
}

impl Problem<TreeChromosome<Op<f32>>, Vec<Tree<Op<f32>>>>
    for Regression<TreeChromosome<Op<f32>>, Vec<Tree<Op<f32>>>>
{
    fn encode(&self) -> Genotype<TreeChromosome<Op<f32>>> {
        self.codex.encode()
    }

    fn decode(&self, genotype: &Genotype<TreeChromosome<Op<f32>>>) -> Vec<Tree<Op<f32>>> {
        self.codex.decode(genotype)
    }

    fn eval(&self, individual: &Genotype<TreeChromosome<Op<f32>>>) -> Score {
        let chrome = individual
            .iter()
            .map(|chrom| chrom.root())
            .collect::<Vec<&TreeNode<Op<f32>>>>();
        self.loss
            .calculate(&self.data_set, &mut |input| chrome.eval(input))
            .into()
    }
}

impl Problem<TreeChromosome<Op<f32>>, Tree<Op<f32>>>
    for Regression<TreeChromosome<Op<f32>>, Tree<Op<f32>>>
{
    fn encode(&self) -> Genotype<TreeChromosome<Op<f32>>> {
        self.codex.encode()
    }

    fn decode(&self, genotype: &Genotype<TreeChromosome<Op<f32>>>) -> Tree<Op<f32>> {
        self.codex.decode(genotype)
    }

    fn eval(&self, individual: &Genotype<TreeChromosome<Op<f32>>>) -> Score {
        let chrome = individual
            .iter()
            .map(|chrom| chrom.root())
            .collect::<Vec<&TreeNode<Op<f32>>>>();
        self.loss
            .calculate(&self.data_set, &mut |input| chrome.eval(input))
            .into()
    }
}

impl Eval<Graph<Op<f32>>, f32> for Regression<GraphChromosome<Op<f32>>, Graph<Op<f32>>> {
    fn eval(&self, graph: &Graph<Op<f32>>) -> f32 {
        let mut evaluator = GraphEvaluator::new(graph);

        self.loss
            .calculate(&self.data_set, &mut |input| evaluator.eval_mut(input))
    }
}

impl Eval<GraphChromosome<Op<f32>>, f32> for Regression<GraphChromosome<Op<f32>>, Graph<Op<f32>>> {
    fn eval(&self, chromosome: &GraphChromosome<Op<f32>>) -> f32 {
        let mut evaluator = GraphEvaluator::new(&chromosome);

        self.loss
            .calculate(&self.data_set, &mut |input| evaluator.eval_mut(input))
    }
}

impl Eval<Tree<Op<f32>>, f32> for Regression<TreeChromosome<Op<f32>>, Vec<Tree<Op<f32>>>> {
    fn eval(&self, tree: &Tree<Op<f32>>) -> f32 {
        self.loss
            .calculate(&self.data_set, &mut |input| vec![tree.eval(input)])
    }
}

impl Eval<Vec<Tree<Op<f32>>>, f32> for Regression<TreeChromosome<Op<f32>>, Vec<Tree<Op<f32>>>> {
    fn eval(&self, program: &Vec<Tree<Op<f32>>>) -> f32 {
        self.loss
            .calculate(&self.data_set, &mut |input| program.eval(input))
    }
}

unsafe impl<C: Chromosome, T: Clone> Send for Regression<C, T> {}
unsafe impl<C: Chromosome, T: Clone> Sync for Regression<C, T> {}
