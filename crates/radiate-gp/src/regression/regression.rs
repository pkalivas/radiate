use super::{DataSet, Loss};
use crate::{Graph, GraphChromosome, GraphEvaluator, Op, Tree, TreeChromosome, eval::EvalIntoMut};
use radiate_core::{BatchFitnessFunction, Genotype, fitness::FitnessFunction};
use std::cell::RefCell;

thread_local! {
    static LOSS_BUFFER: RefCell<Vec<f32>> = RefCell::new(Vec::new());
}

#[derive(Clone)]
pub struct Regression {
    data_set: DataSet<f32>,
    loss: Loss,
}

impl Regression {
    pub fn new(sample_set: impl Into<DataSet<f32>>, loss: Loss) -> Self {
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

// use super::{DataSet, Loss};
// use crate::{Graph, GraphChromosome, GraphEvaluator, Op, Tree, TreeChromosome, eval::EvalIntoMut};
// use radiate_core::{BatchFitnessFunction, Genotype, fitness::FitnessFunction};
// use radiate_utils::LruCache;
// use std::collections::hash_map::DefaultHasher;
// use std::{
//     cell::RefCell,
//     hash::{Hash, Hasher},
// };

// struct Cache {
//     buffer: Vec<f32>,
//     scores: LruCache<u64, f32>,
// }

// thread_local! {
//     static CACHE: RefCell<Cache> = RefCell::new(Cache {
//         buffer: Vec::new(),
//         scores: LruCache::with_capacity(500),
//     });
// }

// #[derive(Clone)]
// pub struct Regression {
//     data_set: DataSet<f32>,
//     loss: Loss,
// }

// impl Regression {
//     pub fn new(sample_set: impl Into<DataSet<f32>>, loss: Loss) -> Self {
//         Regression {
//             data_set: sample_set.into(),
//             loss,
//         }
//     }

//     #[inline]
//     fn try_get_from_cache_or_eval_single<T, F>(&self, val: T, eval_fn: F) -> f32
//     where
//         T: Hash,
//         F: FnOnce(T, &DataSet<f32>, Loss, &mut Vec<f32>) -> f32,
//     {
//         let hash = {
//             let mut hasher = DefaultHasher::new();
//             val.hash(&mut hasher);
//             hasher.finish()
//         };

//         CACHE.with(|cell| {
//             let mut cache = cell.borrow_mut();

//             if let Some(cached) = cache.scores.get(&hash) {
//                 return *cached;
//             }

//             let out_len = self.data_set.shape().2;
//             if cache.buffer.len() < out_len {
//                 cache.buffer.resize(out_len, 0.0);
//             }

//             let result = eval_fn(val, &self.data_set, self.loss, &mut cache.buffer);
//             cache.scores.insert(hash, result);
//             result
//         })
//     }

//     #[inline]
//     fn try_get_from_cache_or_eval_many<T, F>(&self, vals: Vec<T>, eval_fn: F) -> Vec<f32>
//     where
//         T: Hash,
//         F: Fn(T, &DataSet<f32>, Loss, &mut Vec<f32>) -> f32,
//     {
//         let mut results = Vec::with_capacity(vals.len());

//         CACHE.with(|cell| {
//             let mut cache = cell.borrow_mut();

//             let out_len = self.data_set.shape().2;
//             if cache.buffer.len() < out_len {
//                 cache.buffer.resize(out_len, 0.0);
//             }

//             for val in vals {
//                 let hash = {
//                     let mut hasher = DefaultHasher::new();
//                     val.hash(&mut hasher);
//                     hasher.finish()
//                 };

//                 if let Some(cached) = cache.scores.get(&hash) {
//                     results.push(*cached);
//                     continue;
//                 }

//                 let result = eval_fn(val, &self.data_set, self.loss, &mut cache.buffer);
//                 cache.scores.insert(hash, result);
//                 results.push(result);
//             }
//         });

//         results
//     }
// }

// /// --- Graphs ---
// impl<'a> FitnessFunction<&'a Genotype<GraphChromosome<Op<f32>>>, f32> for Regression {
//     #[inline]
//     fn evaluate(&self, input: &'a Genotype<GraphChromosome<Op<f32>>>) -> f32 {
//         self.try_get_from_cache_or_eval_single(input, |val, data_set, loss, buffer| {
//             let mut evaluator = GraphEvaluator::new(&val[0]);
//             loss.calculate(data_set, buffer, |x, y| evaluator.eval_into_mut(x, y))
//         })
//     }
// }

// impl FitnessFunction<Graph<Op<f32>>, f32> for Regression {
//     #[inline]
//     fn evaluate(&self, input: Graph<Op<f32>>) -> f32 {
//         self.try_get_from_cache_or_eval_single(input, |val, data_set, loss, buffer| {
//             let mut evaluator = GraphEvaluator::new(&val);
//             loss.calculate(data_set, buffer, |x, y| evaluator.eval_into_mut(x, y))
//         })
//     }
// }

// impl BatchFitnessFunction<Graph<Op<f32>>, f32> for Regression {
//     #[inline]
//     fn evaluate(&self, inputs: Vec<Graph<Op<f32>>>) -> Vec<f32> {
//         self.try_get_from_cache_or_eval_many(inputs, |val, data_set, loss, buffer| {
//             let mut evaluator = GraphEvaluator::new(&val);
//             loss.calculate(data_set, buffer, |x, y| evaluator.eval_into_mut(x, y))
//         })
//     }
// }

// impl<'a> BatchFitnessFunction<&'a Genotype<GraphChromosome<Op<f32>>>, f32> for Regression {
//     #[inline]
//     fn evaluate(&self, inputs: Vec<&'a Genotype<GraphChromosome<Op<f32>>>>) -> Vec<f32> {
//         self.try_get_from_cache_or_eval_many(inputs, |val, data_set, loss, buffer| {
//             let mut evaluator = GraphEvaluator::new(&val[0]);
//             loss.calculate(data_set, buffer, |x, y| evaluator.eval_into_mut(x, y))
//         })
//     }
// }

// /// --- Trees ---
// impl FitnessFunction<Tree<Op<f32>>, f32> for Regression {
//     #[inline]
//     fn evaluate(&self, input: Tree<Op<f32>>) -> f32 {
//         self.try_get_from_cache_or_eval_single(input, |mut val, data_set, loss, buffer| {
//             loss.calculate(data_set, buffer, |x, y| val.eval_into_mut(x, y))
//         })
//     }
// }

// impl FitnessFunction<Vec<Tree<Op<f32>>>, f32> for Regression {
//     #[inline]
//     fn evaluate(&self, input: Vec<Tree<Op<f32>>>) -> f32 {
//         self.try_get_from_cache_or_eval_single(input, |mut vals, data_set, loss, buffer| {
//             loss.calculate(data_set, buffer, |x, y| vals.eval_into_mut(x, y))
//         })
//     }
// }

// impl BatchFitnessFunction<Tree<Op<f32>>, f32> for Regression {
//     #[inline]
//     fn evaluate(&self, inputs: Vec<Tree<Op<f32>>>) -> Vec<f32> {
//         self.try_get_from_cache_or_eval_many(inputs, |mut vals, data_set, loss, buffer| {
//             loss.calculate(data_set, buffer, |x, y| vals.eval_into_mut(x, y))
//         })
//     }
// }

// impl BatchFitnessFunction<Vec<Tree<Op<f32>>>, f32> for Regression {
//     #[inline]
//     fn evaluate(&self, inputs: Vec<Vec<Tree<Op<f32>>>>) -> Vec<f32> {
//         self.try_get_from_cache_or_eval_many(inputs, |mut vals, data_set, loss, buffer| {
//             loss.calculate(data_set, buffer, |x, y| vals.eval_into_mut(x, y))
//         })
//     }
// }

// impl<'a> FitnessFunction<&'a Genotype<TreeChromosome<Op<f32>>>, f32> for Regression {
//     #[inline]
//     fn evaluate(&self, input: &'a Genotype<TreeChromosome<Op<f32>>>) -> f32 {
//         let roots = input.iter().map(|c| c.root()).collect::<Vec<_>>();
//         let roots_ref = roots.as_slice();
//         self.try_get_from_cache_or_eval_single(roots_ref, |mut val, data_set, loss, buffer| {
//             loss.calculate(data_set, buffer, |x, y| val.eval_into_mut(x, y))
//         })
//     }
// }

// impl<'a> BatchFitnessFunction<&'a Genotype<TreeChromosome<Op<f32>>>, f32> for Regression {
//     #[inline]
//     fn evaluate(&self, inputs: Vec<&'a Genotype<TreeChromosome<Op<f32>>>>) -> Vec<f32> {
//         self.try_get_from_cache_or_eval_many(inputs, |vals, data_set, loss, buffer| {
//             let roots = vals.iter().map(|c| c.root()).collect::<Vec<_>>();
//             let mut roots_ref = roots.as_slice();
//             loss.calculate(data_set, buffer, |x, y| roots_ref.eval_into_mut(x, y))
//         })
//     }
// }
