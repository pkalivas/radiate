use super::{DataSet, Loss};
use crate::graphs::compile::{
    BatchMode::{self, Generic},
    BatchState, BatchStats, GraphKernel,
};
use crate::{GraphChromosome, GraphEvaluator, Op, TreeChromosome, eval::EvalIntoMut, ops::OpFloat};
use crate::{GraphNode, graphs::compile::BatchMode::Kernels};
use radiate_core::{BatchFitnessFunction, Genotype, Score};
use radiate_utils::Matrix;
use std::sync::OnceLock;

#[derive(Clone)]
pub struct Regression<F: OpFloat> {
    data_set: DataSet<F>,
    loss: Loss,
    /// The features transposed to one row per feature, for column-wise graph evaluation.
    /// Built on first use, so row-only evaluation never pays for the copy.
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
    #[inline]
    fn evaluate(&self, inputs: &[&'a Genotype<GraphChromosome<Op<F>>>]) -> Vec<F> {
        self.with_buff(move |buf| {
            let mut results = Vec::with_capacity(inputs.len());
            let mode = Some(Kernels);
            let mut compiled = GraphKernel::default();
            let mut state = BatchState::default();
            let mut stats = BatchStats::default();
            for input in inputs.iter() {
                let nodes = input[0].as_ref();
                results.push(self.graph_loss_buf(
                    nodes,
                    mode,
                    buf,
                    &mut compiled,
                    &mut state,
                    &mut stats,
                ));
            }

            results
        })
    }
}

impl<F: OpFloat> Regression<F> {
    fn graph_loss_buf(
        &self,
        nodes: &[GraphNode<Op<F>>],
        mode: Option<BatchMode>,
        buf: &mut [F],
        compiled: &mut GraphKernel,
        state: &mut BatchState<F>,
        stats: &mut BatchStats,
    ) -> F {
        if let Some(mode) = mode {
            compiled.compile(nodes);
            compiled.eval_batch(nodes, self.feature_cols(), state, mode, stats);
            // println!("{:#?}", compiled);

            // panic!("Compiled graph evaluation not implemented yet");
            return self.loss.calculate_columns(
                self.data_set.label_matrix(),
                state.values(),
                compiled.output_slots(),
            );
        }

        let mut evaluator = GraphEvaluator::new(&nodes);
        self.calc_with_buf(&mut evaluator, buf)
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

/// PROTOTYPE switch for graph evaluation, read once from the environment:
/// `RADIATE_COL=0` (row evaluator, default) | `1` (column, generic) | `2` (column, kernels).
pub fn col_mode() -> Option<BatchMode> {
    static MODE: OnceLock<Option<BatchMode>> = OnceLock::new();
    *MODE.get_or_init(|| match std::env::var("RADIATE_COL").as_deref() {
        Ok("1") => Some(BatchMode::Generic),
        Ok("2") => Some(BatchMode::Kernels),
        _ => None,
    })
}
