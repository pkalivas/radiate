use super::{DataSet, Loss};
use crate::{
    Graph, GraphChromosome, GraphEvaluator, Op, Tree, TreeChromosome, eval::EvalIntoMut,
    ops::OpFloat,
};
use radiate_core::{BatchFitnessFunction, Genotype, Score, fitness::FitnessFunction};

#[derive(Clone)]
pub struct Regression<F: OpFloat> {
    data_set: DataSet<F>,
    loss: Loss,
}

impl<F: OpFloat> Regression<F> {
    pub fn new(sample_set: impl Into<DataSet<F>>, loss: Loss) -> Self {
        Regression {
            data_set: sample_set.into(),
            loss,
        }
    }

    #[inline]
    fn calc_into_buff_mut<EV>(&self, eval: &mut EV) -> F
    where
        EV: EvalIntoMut<[F], [F]>,
    {
        let out_len = self.data_set.shape().2;
        F::with_loss_buffer(|buf| {
            if buf.len() < out_len {
                buf.resize(out_len, F::ZERO);
            }

            self.loss
                .calculate(&self.data_set, &mut buf[..out_len], |x, y| {
                    eval.eval_into_mut(x, y)
                })
        })
    }
}

impl<'a, F> FitnessFunction<&'a Genotype<GraphChromosome<Op<F>>>, F> for Regression<F>
where
    F: OpFloat + Into<Score>,
{
    #[inline]
    fn evaluate(&self, input: &'a Genotype<GraphChromosome<Op<F>>>) -> F {
        let mut evaluator = GraphEvaluator::new(&input[0]);
        self.calc_into_buff_mut(&mut evaluator)
    }
}

impl<F> FitnessFunction<Graph<Op<F>>, F> for Regression<F>
where
    F: OpFloat + Into<Score>,
{
    #[inline]
    fn evaluate(&self, input: Graph<Op<F>>) -> F {
        let mut evaluator = GraphEvaluator::new(&input);
        self.calc_into_buff_mut(&mut evaluator)
    }
}

impl<F> BatchFitnessFunction<Graph<Op<F>>, F> for Regression<F>
where
    F: OpFloat + Into<Score>,
{
    #[inline]
    fn evaluate(&self, inputs: Vec<Graph<Op<F>>>) -> Vec<F> {
        let mut results = Vec::with_capacity(inputs.len());
        for input in inputs {
            let mut evaluator = GraphEvaluator::new(&input);
            results.push(self.calc_into_buff_mut(&mut evaluator));
        }

        results
    }
}

impl<'a, F> BatchFitnessFunction<&'a Genotype<GraphChromosome<Op<F>>>, F> for Regression<F>
where
    F: OpFloat + Into<Score>,
{
    #[inline]
    fn evaluate(&self, inputs: Vec<&'a Genotype<GraphChromosome<Op<F>>>>) -> Vec<F> {
        let mut results = Vec::with_capacity(inputs.len());
        for input in inputs {
            let mut evaluator = GraphEvaluator::new(&input[0]);
            results.push(self.calc_into_buff_mut(&mut evaluator));
        }

        results
    }
}

/// --- Trees ---
impl<F> FitnessFunction<Tree<Op<F>>, F> for Regression<F>
where
    F: OpFloat + Into<Score>,
{
    #[inline]
    fn evaluate(&self, mut input: Tree<Op<F>>) -> F {
        self.calc_into_buff_mut(&mut input)
    }
}

impl<F> FitnessFunction<Vec<Tree<Op<F>>>, F> for Regression<F>
where
    F: OpFloat + Into<Score>,
{
    #[inline]
    fn evaluate(&self, mut input: Vec<Tree<Op<F>>>) -> F {
        self.calc_into_buff_mut(&mut input)
    }
}

impl<F> BatchFitnessFunction<Tree<Op<F>>, F> for Regression<F>
where
    F: OpFloat + Into<Score>,
{
    #[inline]
    fn evaluate(&self, mut inputs: Vec<Tree<Op<F>>>) -> Vec<F> {
        let mut results = Vec::with_capacity(inputs.len());
        for input in inputs.iter_mut() {
            results.push(self.calc_into_buff_mut(input));
        }

        results
    }
}

impl<F> BatchFitnessFunction<Vec<Tree<Op<F>>>, F> for Regression<F>
where
    F: OpFloat + Into<Score>,
{
    #[inline]
    fn evaluate(&self, mut inputs: Vec<Vec<Tree<Op<F>>>>) -> Vec<F> {
        let mut results = Vec::with_capacity(inputs.len());
        for input in inputs.iter_mut() {
            results.push(self.calc_into_buff_mut(input));
        }

        results
    }
}

impl<'a, F> FitnessFunction<&'a Genotype<TreeChromosome<Op<F>>>, F> for Regression<F>
where
    F: OpFloat + Into<Score>,
{
    #[inline]
    fn evaluate(&self, input: &'a Genotype<TreeChromosome<Op<F>>>) -> F {
        let roots = input.iter().map(|c| c.root()).collect::<Vec<_>>();
        self.calc_into_buff_mut(&mut roots.as_slice())
    }
}

impl<'a, F> BatchFitnessFunction<&'a Genotype<TreeChromosome<Op<F>>>, F> for Regression<F>
where
    F: OpFloat + Into<Score>,
{
    #[inline]
    fn evaluate(&self, inputs: Vec<&'a Genotype<TreeChromosome<Op<F>>>>) -> Vec<F> {
        let mut results = Vec::with_capacity(inputs.len());
        for input in inputs {
            let roots = input.iter().map(|c| c.root()).collect::<Vec<_>>();
            results.push(self.calc_into_buff_mut(&mut roots.as_slice()));
        }

        results
    }
}
