use super::{DataSet, Loss};
use crate::{
    GraphChromosome, GraphEvaluator, Op, TreeChromosome, eval::EvalIntoMut, graphs::GraphEvalCache,
    ops::OpFloat,
};
use radiate_core::{BatchFitnessFunction, Genotype, Score};
use radiate_utils::Matrix;
use std::sync::OnceLock;

#[derive(Clone)]
pub struct Regression<F: OpFloat> {
    data_set: DataSet<F>,
    loss: Loss,
    feature_cols: OnceLock<Matrix<F>>,
}

impl<F: OpFloat> Regression<F> {
    pub fn new(sample_set: impl Into<DataSet<F>>, loss: Loss) -> Self {
        Regression {
            data_set: sample_set.into(),
            loss,
            feature_cols: OnceLock::new(),
        }
    }

    #[inline]
    fn feature_cols(&self) -> &Matrix<F> {
        self.feature_cols
            .get_or_init(|| self.data_set.feature_matrix().transpose())
    }

    #[inline]
    fn calc_with_buf<EV>(&self, eval: &mut EV, buf: &mut [F]) -> F
    where
        EV: EvalIntoMut<[F], [F]>,
    {
        self.loss
            .calculate(&self.data_set, buf, |x, y| eval.eval_into_mut(x, y))
    }

    fn with_buff<E, O>(&self, eval_fn: E) -> O
    where
        E: FnOnce(&mut [F]) -> O,
    {
        let out_len = self.data_set.shape().2;
        F::with_loss_buffer(|buf| {
            if buf.len() < out_len {
                buf.resize(out_len, F::ZERO);
            }

            eval_fn(&mut buf[..out_len])
        })
    }
}

impl<'a, F> BatchFitnessFunction<&'a Genotype<GraphChromosome<Op<F>>>, F> for Regression<F>
where
    F: OpFloat + Into<Score>,
{
    /// Evaluates each graph over the whole data set at once with a [GraphKernel], carrying one
    /// [KernelCache] from graph to graph so the batch reuses its allocations.
    #[inline]
    fn evaluate(&self, inputs: &[&'a Genotype<GraphChromosome<Op<F>>>]) -> Vec<F> {
        let features = self.feature_cols();
        let labels = self.data_set.label_matrix();

        let mut cache = GraphEvalCache::default();
        let mut preds = Matrix::default();
        let mut results = Vec::with_capacity(inputs.len());
        for input in inputs.iter() {
            let mut kernel = GraphEvaluator::from((&input[0], cache));
            kernel.eval_into_mut(features, &mut preds);
            cache = kernel.take_cache();
            cache.bump();

            results.push(self.loss.calculate_columns(labels, &preds));
        }

        results
    }
}

impl<'a, F> BatchFitnessFunction<&'a Genotype<TreeChromosome<Op<F>>>, F> for Regression<F>
where
    F: OpFloat + Into<Score>,
{
    #[inline]
    fn evaluate(&self, inputs: &[&'a Genotype<TreeChromosome<Op<F>>>]) -> Vec<F> {
        self.with_buff(move |buf| {
            let mut results = Vec::with_capacity(inputs.len());
            for input in inputs.iter() {
                let roots = input.iter().map(|c| c.root()).collect::<Vec<_>>();
                results.push(self.calc_with_buf(&mut roots.as_slice(), buf));
            }

            results
        })
    }
}
