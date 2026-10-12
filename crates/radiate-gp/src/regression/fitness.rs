use super::Loss;
use crate::{
    GraphChromosome, GraphEvaluator, Op, TreeChromosome, eval::EvalIntoMut, graphs::GraphEvalCache,
    ops::OpFloat, trees::TreeEvalCache,
};
use radiate_core::{BatchFitnessFunction, Genotype, Score};
use radiate_utils::{DataSet, Matrix};
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
    fn features_matrix_t(&self) -> &Matrix<F> {
        self.feature_cols
            .get_or_init(|| self.data_set.feature_matrix().transpose())
    }
}

impl<'a, F> BatchFitnessFunction<&'a Genotype<GraphChromosome<Op<F>>>, F> for Regression<F>
where
    F: OpFloat + Into<Score>,
{
    #[inline]
    fn evaluate(&self, graphs: Vec<&Genotype<GraphChromosome<Op<F>>>>) -> Vec<F> {
        let features = self.features_matrix_t();
        let labels = self.data_set.label_matrix();

        let mut cache = GraphEvalCache::default();
        let mut preds = Matrix::default();
        let mut scores = Vec::with_capacity(graphs.len());

        for graph in graphs.iter() {
            let mut evaluator = GraphEvaluator::with_cache(&graph[0], cache);
            evaluator.eval_into_mut(features, &mut preds);
            cache = evaluator.take_cache();

            scores.push(self.loss.calculate_columns(labels, &preds));
        }

        scores
    }
}

impl<'a, F> BatchFitnessFunction<&'a Genotype<TreeChromosome<Op<F>>>, F> for Regression<F>
where
    F: OpFloat + Into<Score>,
{
    #[inline]
    fn evaluate(&self, trees: Vec<&Genotype<TreeChromosome<Op<F>>>>) -> Vec<F> {
        let features = self.features_matrix_t();
        let labels = self.data_set.label_matrix();

        let mut cache = TreeEvalCache::default();
        let mut preds = Matrix::default();
        let mut roots = Vec::new();
        let mut scores = Vec::with_capacity(trees.len());

        for genotype in trees.iter() {
            roots.clear();
            roots.extend(genotype.iter().map(|c| c.root()));
            cache.eval_columns(&roots, features, &mut preds);

            scores.push(self.loss.calculate_columns(labels, &preds));
        }

        scores
    }
}
