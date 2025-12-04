use super::{DataSet, Loss};
use crate::{Graph, GraphChromosome, GraphCodec, GraphEvaluator, Op, Tree, eval::EvalIntoMut};
use radiate_core::{
    BatchFitnessFunction, Codec, Genotype, Problem, RadiateError, Score, fitness::FitnessFunction,
};
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

impl Problem<GraphChromosome<Op<f32>>, Graph<Op<f32>>> for (Regression, GraphCodec<Op<f32>>) {
    fn encode(&self) -> Genotype<GraphChromosome<Op<f32>>> {
        self.1.encode()
    }

    fn decode(&self, genotype: &Genotype<GraphChromosome<Op<f32>>>) -> Graph<Op<f32>> {
        self.1.decode(genotype)
    }

    fn eval(&self, individual: &Genotype<GraphChromosome<Op<f32>>>) -> Result<Score, RadiateError> {
        if individual.len() != 1 {
            return Err(RadiateError::Evaluation(
                "Expected genotype with a single individual.".to_string(),
            ));
        }

        let mut evaluator = GraphEvaluator::new(&individual[0]);
        Ok(Score::from(self.0.calc_into_buff_mut(&mut evaluator)))
    }

    fn eval_batch(
        &self,
        individuals: &[Genotype<GraphChromosome<Op<f32>>>],
    ) -> Result<Vec<Score>, RadiateError> {
        let mut results = Vec::with_capacity(individuals.len());
        for individual in individuals {
            if individual.len() != 1 {
                return Err(RadiateError::Evaluation(
                    "Expected genotype with a single individual.".to_string(),
                ));
            }

            let mut evaluator = GraphEvaluator::new(&individual[0]);
            results.push(Score::from(self.0.calc_into_buff_mut(&mut evaluator)));
        }

        Ok(results)
    }
}
