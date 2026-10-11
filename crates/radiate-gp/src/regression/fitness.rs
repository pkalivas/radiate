use super::{DataSet, Loss};
use crate::GraphNode;
use crate::graphs::column::{ColBuf, ColMode, ColStats, eval_columns};
use crate::{GraphChromosome, GraphEvaluator, Op, TreeChromosome, eval::EvalIntoMut, ops::OpFloat};
use radiate_core::{BatchFitnessFunction, Genotype, Score};
use std::sync::OnceLock;

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
            let mode = col_mode();
            let rows: Vec<&[F]> = self.data_set.iter().map(|(x, _)| x).collect();
            let mut cbuf = ColBuf::default();
            let mut stats = ColStats::default();
            for input in inputs.iter() {
                let nodes = input[0].as_ref();
                results.push(self.graph_loss_buf(nodes, mode, &rows, buf, &mut cbuf, &mut stats));
            }

            results
        })
    }
}

/// PROTOTYPE switch for graph evaluation, read once from the environment:
/// `RADIATE_COL=0` (row evaluator, default) | `1` (column, generic) | `2` (column, kernels).
pub fn col_mode() -> Option<ColMode> {
    static MODE: OnceLock<Option<ColMode>> = OnceLock::new();
    *MODE.get_or_init(|| match std::env::var("RADIATE_COL").as_deref() {
        Ok("1") => Some(ColMode::Generic),
        Ok("2") => Some(ColMode::Kernels),
        _ => None,
    })
}

impl<F: OpFloat> Regression<F> {
    /// PROTOTYPE: loss of one graph with the given evaluation mode (`None` = row evaluator).
    /// Graphs that can't be evaluated column-wise fall back to the row evaluator.
    pub fn graph_loss(
        &self,
        nodes: &[GraphNode<Op<F>>],
        mode: Option<ColMode>,
        cbuf: &mut ColBuf<F>,
        stats: &mut ColStats,
    ) -> F {
        let rows: Vec<&[F]> = self.data_set.iter().map(|(x, _)| x).collect();
        self.with_buff(|buf| self.graph_loss_buf(nodes, mode, &rows, buf, cbuf, stats))
    }

    fn graph_loss_buf(
        &self,
        nodes: &[GraphNode<Op<F>>],
        mode: Option<ColMode>,
        rows: &[&[F]],
        buf: &mut [F],
        cbuf: &mut ColBuf<F>,
        stats: &mut ColStats,
    ) -> F {
        if let Some(mode) = mode
            && eval_columns(nodes, rows, cbuf, mode, stats)
        {
            let n = rows.len();
            let mut r = 0;
            return self.loss.calculate(&self.data_set, buf, |_, y| {
                for (d, &slot) in y.iter_mut().zip(cbuf.output_slots()) {
                    *d = cbuf.value(slot, n, r);
                }
                r += 1;
            });
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
