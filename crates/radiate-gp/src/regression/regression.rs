use super::{DataSet, Loss};
use crate::{Graph, GraphChromosome, GraphEvaluator, Op, Tree, TreeChromosome, eval::EvalIntoMut};
use radiate_core::{BatchFitnessFunction, Genotype, fitness::FitnessFunction};
use std::cell::RefCell;

thread_local! {
    static LOSS_BUFFER: RefCell<Vec<f32>> = RefCell::new(Vec::new());
}

#[derive(Clone)]
pub struct Regression {
    data_set: DataSet,
    loss: Loss,
}

impl Regression {
    pub fn new(sample_set: impl Into<DataSet>, loss: Loss) -> Self {
        Regression {
            data_set: sample_set.into(),
            loss,
        }
    }

    #[inline]
    fn calc_into_buff_mut<EV>(&self, eval: &mut EV) -> f32
    where
        EV: EvalIntoMut<[f32], [f32]>,
    {
        let out_len = self.data_set.shape().2;
        LOSS_BUFFER.with(|cell| {
            let mut buf = cell.borrow_mut();

            if buf.len() < out_len {
                buf.resize(out_len, 0.0);
            }

            self.loss
                .calculate(&self.data_set, &mut buf[..out_len], |x, y| {
                    eval.eval_into_mut(x, y)
                })
        })
    }
}

/// --- Graphs ---
impl<'a> FitnessFunction<&'a Genotype<GraphChromosome<Op<f32>>>, f32> for Regression {
    #[inline]
    fn evaluate(&self, input: &'a Genotype<GraphChromosome<Op<f32>>>) -> f32 {
        let mut evaluator = GraphEvaluator::new(&input[0]);
        self.calc_into_buff_mut(&mut evaluator)
    }
}

impl FitnessFunction<Graph<Op<f32>>, f32> for Regression {
    #[inline]
    fn evaluate(&self, input: Graph<Op<f32>>) -> f32 {
        let mut evaluator = GraphEvaluator::new(&input);
        self.calc_into_buff_mut(&mut evaluator)
    }
}

impl BatchFitnessFunction<Graph<Op<f32>>, f32> for Regression {
    #[inline]
    fn evaluate(&self, inputs: Vec<Graph<Op<f32>>>) -> Vec<f32> {
        let mut results = Vec::with_capacity(inputs.len());
        for input in inputs {
            let mut evaluator = GraphEvaluator::new(&input);
            results.push(self.calc_into_buff_mut(&mut evaluator));
        }

        results
    }
}

impl<'a> BatchFitnessFunction<&'a Genotype<GraphChromosome<Op<f32>>>, f32> for Regression {
    #[inline]
    fn evaluate(&self, inputs: Vec<&'a Genotype<GraphChromosome<Op<f32>>>>) -> Vec<f32> {
        let mut results = Vec::with_capacity(inputs.len());
        for input in inputs {
            let mut evaluator = GraphEvaluator::new(&input[0]);
            results.push(self.calc_into_buff_mut(&mut evaluator));
        }

        results
    }
}

/// --- Trees ---
impl FitnessFunction<Tree<Op<f32>>, f32> for Regression {
    #[inline]
    fn evaluate(&self, mut input: Tree<Op<f32>>) -> f32 {
        self.calc_into_buff_mut(&mut input)
    }
}

impl FitnessFunction<Vec<Tree<Op<f32>>>, f32> for Regression {
    #[inline]
    fn evaluate(&self, mut input: Vec<Tree<Op<f32>>>) -> f32 {
        self.calc_into_buff_mut(&mut input)
    }
}

impl BatchFitnessFunction<Tree<Op<f32>>, f32> for Regression {
    #[inline]
    fn evaluate(&self, mut inputs: Vec<Tree<Op<f32>>>) -> Vec<f32> {
        let mut results = Vec::with_capacity(inputs.len());
        for input in inputs.iter_mut() {
            results.push(self.calc_into_buff_mut(input));
        }

        results
    }
}

impl BatchFitnessFunction<Vec<Tree<Op<f32>>>, f32> for Regression {
    #[inline]
    fn evaluate(&self, mut inputs: Vec<Vec<Tree<Op<f32>>>>) -> Vec<f32> {
        let mut results = Vec::with_capacity(inputs.len());
        for input in inputs.iter_mut() {
            results.push(self.calc_into_buff_mut(input));
        }

        results
    }
}

impl<'a> FitnessFunction<&'a Genotype<TreeChromosome<Op<f32>>>, f32> for Regression {
    #[inline]
    fn evaluate(&self, input: &'a Genotype<TreeChromosome<Op<f32>>>) -> f32 {
        let roots = input.iter().map(|c| c.root()).collect::<Vec<_>>();
        self.calc_into_buff_mut(&mut roots.as_slice())
    }
}

impl<'a> BatchFitnessFunction<&'a Genotype<TreeChromosome<Op<f32>>>, f32> for Regression {
    #[inline]
    fn evaluate(&self, inputs: Vec<&'a Genotype<TreeChromosome<Op<f32>>>>) -> Vec<f32> {
        let mut results = Vec::with_capacity(inputs.len());
        for input in inputs {
            let roots = input.iter().map(|c| c.root()).collect::<Vec<_>>();
            results.push(self.calc_into_buff_mut(&mut roots.as_slice()));
        }

        results
    }
}
