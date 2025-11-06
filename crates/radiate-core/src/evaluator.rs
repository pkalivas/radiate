//! # Fitness Evaluators
//!
//! This module provides fitness evaluation strategies for genetic algorithms, offering
//! both individual and batch evaluation approaches. Evaluators are responsible for
//! computing fitness scores for individuals in the population using the provided problem.
//!
//! The module provides two main evaluation strategies:
//! - **FitnessEvaluator**: Evaluates individuals one at a time
//! - **BatchFitnessEvaluator**: Evaluates individuals in batches for better performance
//!
//! Both evaluators support parallel execution through the executor system and integrate
//! seamlessly with the ecosystem and problem abstractions.

use crate::Result;
use crate::{Chromosome, Ecosystem, Executor, Problem};
use std::sync::Arc;

/// A trait for evaluating fitness of individuals in the ecosystem.
///
/// The [Evaluator] trait defines the interface for fitness evaluation strategies.
/// Implementors can define different approaches to computing fitness scores,
/// such as individual evaluation, batch evaluation, or specialized evaluation
/// strategies for specific problem domains.
///
/// The two main implementations provided are:
/// - [FitnessEvaluator]: Evaluates individuals one at a time
/// - [BatchFitnessEvaluator]: Evaluates individuals in batches
/// Custom evaluators can be created and used, however, take special note on how the
/// members of the ecosystem are accessed and modified, (taken out of the phenotype then restored).
/// This is important to ensure memory safety and avoid unnecessary cloning of genotypes.
///
/// # Generic Parameters
/// - `C`: The chromosome type that represents the genetic material
/// - `T`: The target type that the problem operates on
pub trait Evaluator<C: Chromosome, T>: Send + Sync {
    /// Evaluates the fitness of unevaluated individuals in the ecosystem.
    ///
    /// This method processes individuals that don't have fitness scores yet,
    /// evaluates them using the provided problem, and updates the ecosystem
    /// with the computed scores.
    ///
    /// # Arguments
    ///
    /// * `ecosystem` - The ecosystem containing the population
    /// * `problem` - The problem instance used to evaluate fitness
    ///
    /// # Returns
    ///
    /// The number of individuals that were evaluated during this call
    fn eval(&self, ecosystem: &mut Ecosystem<C>, problem: Arc<dyn Problem<C, T>>) -> Result<usize>;
}

/// A fitness evaluator that evaluates individuals one at a time.
///
/// `FitnessEvaluator` processes individuals individually, which is suitable for
/// problems where individual evaluation is the most efficient approach or when
/// you need fine-grained control over the evaluation process.
///
/// # Performance Characteristics
///
/// - **Individual Processing**: Each individual is evaluated separately
/// - **Parallelization**: Can utilize multiple threads through the executor
/// - **Flexibility**: Easy to implement custom evaluation logic
pub struct FitnessEvaluator {
    executor: Arc<Executor>,
}

impl FitnessEvaluator {
    /// Creates a new fitness evaluator with the specified executor.
    ///
    /// # Arguments
    ///
    /// * `executor` - The executor to use for running fitness evaluations
    ///
    /// # Returns
    ///
    /// A new `FitnessEvaluator` instance
    ///
    /// # Note
    ///
    /// Choose an executor that matches your performance requirements:
    /// - [Executor::Serial] for single-threaded execution
    /// - [Executor::ThreadPool(n)] for parallel execution with n worker threads
    pub fn new(executor: Arc<Executor>) -> Self {
        Self { executor }
    }
}

/// Implementation of [Evaluator] for [FitnessEvaluator].
///
/// This implementation evaluates individuals one at a time, collecting unevaluated
/// individuals and processing them through the executor system.
///
/// # Algorithm
///
/// 1. **Collect Unevaluated Individuals**: Find individuals without fitness scores
/// 2. **Create Evaluation Jobs**: Package genotypes and indices into jobs
/// 3. **Execute Jobs**: Run evaluations through the executor
/// 4. **Update Ecosystem**: Store computed scores and restore genotypes
///
/// # Performance Optimizations
///
/// - **Efficient Job Creation**: Jobs are created with minimal allocation
/// - **Batch Execution**: Multiple jobs are submitted to the executor at once
/// - **Memory Reuse**: Genotypes are restored to avoid unnecessary cloning
impl<C: Chromosome, T> Evaluator<C, T> for FitnessEvaluator
where
    C: Chromosome + 'static,
    T: 'static,
{
    #[inline]
    fn eval(&self, ecosystem: &mut Ecosystem<C>, problem: Arc<dyn Problem<C, T>>) -> Result<usize> {
        let mut jobs = Vec::new();
        let len = ecosystem.population.len();
        for idx in 0..len {
            if ecosystem.population[idx].score().is_none() {
                let geno = ecosystem.population[idx].take_genotype()?;
                jobs.push((idx, geno));
            }
        }

        let results = self.executor.execute_batch(
            jobs.into_iter()
                .map(|(idx, geno)| {
                    let problem = Arc::clone(&problem);
                    move || {
                        let score = problem.eval(&geno);
                        (idx, score, geno)
                    }
                })
                .collect::<Vec<_>>(),
        );

        let count = results.len();
        for result in results {
            let (idx, score, genotype) = result;
            ecosystem.population[idx].set_score(Some(score?));
            ecosystem.population[idx].set_genotype(genotype);
        }

        Ok(count)
    }
}

/// Default implementation for [FitnessEvaluator].
///
/// Creates a fitness evaluator with a serial executor, which is suitable for
/// single-threaded applications or when parallel execution is not needed.
///
/// # Note
///
/// The default executor is [Executor::Serial], which processes evaluations
/// sequentially. For better performance with large populations, consider using
/// [variant@Executor::ThreadPool(n)] with an appropriate number of worker threads
/// or [Executor::WorkerPool] (rayon feature required).
impl Default for FitnessEvaluator {
    fn default() -> Self {
        Self {
            executor: Arc::new(Executor::Serial),
        }
    }
}

/// A fitness evaluator that evaluates individuals in batches for improved performance or
/// for when you need access to parts or the whole of an unevaluated population in order
/// to compute fitness.
///
/// [BatchFitnessEvaluator] groups individuals into batches and evaluates them
/// together, which can be more efficient than individual evaluation, especially
/// when the problem supports batch evaluation or when there are significant
/// overhead costs per evaluation.
///
/// # Performance Characteristics
///
/// - **Batch Processing**: Multiple individuals evaluated together
/// - **Reduced Overhead**: Sometimes lower per-individual evaluation cost
///
/// # Use Cases
///
/// Use [BatchFitnessEvaluator] when:
/// - The problem supports efficient batch evaluation
/// - You have a large population that benefits from parallelization
/// - Individual evaluation overhead is significant
/// - You need access to parts or the whole of an unevaluated population in order to compute fitness.
///
/// # Batch Size Calculation
///
/// In order to ensure the optimal distribution of work across available threads,
/// batch size is automatically calculated based on the number of workers
/// in the executor and the total number of individuals to evaluate:
///
/// ```text
/// batch_size = (total_individuals + num_workers - 1) / num_workers
/// ```
/// So, for example, if you have 100 individuals and 4 workers, the batch size would be:
///
/// ```text
/// batch_size = (100 + 4 - 1) / 4 = 25
/// ```
///
/// But, if your executor is [Executor::Serial], then the batch size will simply be the total number of
/// individuals to evaluate, resulting in a single batch.
pub struct BatchFitnessEvaluator {
    executor: Arc<Executor>,
}

impl BatchFitnessEvaluator {
    /// Creates a new batch fitness evaluator with the specified executor.
    ///
    /// # Arguments
    ///
    /// * `executor` - The executor to use for running batch evaluations
    ///
    /// # Returns
    ///
    /// A new [BatchFitnessEvaluator] instance
    ///
    /// # Note
    ///
    /// Batch evaluation works best with parallel executors that have multiple
    /// worker threads. The evaluator will automatically divide work into optimal
    /// batch sizes based on the number of available workers.
    pub fn new(executor: Arc<Executor>) -> Self {
        Self { executor }
    }
}

/// Implementation of [Evaluator] for [BatchFitnessEvaluator].
///
/// This implementation groups individuals into batches and evaluates them
/// together, which can significantly improve performance for large populations
/// or when the problem supports efficient batch evaluation.
///
/// # Algorithm
///
/// The algorithm largely follows the same steps as `FitnessEvaluator`:
/// 1. **Collect Unevaluated Individuals**: Find individuals without fitness scores
/// 2. **Calculate Batch Size**: Determine optimal batch size based on worker count
/// 3. **Create Batches**: Group individuals into batches for parallel processing
/// 4. **Execute Batches**: Run batch evaluations through the executor
/// 5. **Update Ecosystem**: Store computed scores and restore genotypes
///
/// # Batch Size Optimization
///
/// The batch size is calculated to ensure optimal work distribution:
/// - **Small Batches**: Ensure all workers have work to do
/// - **Large Batches**: Minimize overhead from job creation and distribution
/// - **Balanced Distribution**: Work is evenly distributed across available threads
///
/// # Performance Optimizations
///
/// - **Efficient Batching**: Batches are created with minimal memory allocation
/// - **Parallel Execution**: Multiple batches are processed simultaneously
/// - **Memory Management**: Genotypes are efficiently restored after evaluation
impl<C: Chromosome, T> Evaluator<C, T> for BatchFitnessEvaluator
where
    C: Chromosome + 'static,
    T: 'static,
{
    #[inline]
    fn eval(&self, ecosystem: &mut Ecosystem<C>, problem: Arc<dyn Problem<C, T>>) -> Result<usize> {
        let mut pairs = Vec::new();
        let len = ecosystem.population.len();
        for idx in 0..len {
            if ecosystem.population[idx].score().is_none() {
                let geno = ecosystem.population[idx].take_genotype()?;
                pairs.push((idx, geno));
            }
        }

        let num_workers = self.executor.num_workers();
        let batch_size = (pairs.len() + num_workers - 1) / num_workers;

        if pairs.is_empty() || batch_size == 0 {
            return Ok(0);
        }

        let mut batches = Vec::with_capacity(num_workers);

        while !pairs.is_empty() {
            let take = pairs.len().min(batch_size);

            let mut batch_indices = Vec::with_capacity(take);
            let mut batch_genotypes = Vec::with_capacity(take);

            // drain from the end of pairs vector to avoid O(n^2) complexity
            for (idx, geno) in pairs.drain(pairs.len() - take..) {
                batch_indices.push(idx);
                batch_genotypes.push(geno);
            }

            batches.push((batch_indices, batch_genotypes));
        }

        let results = self.executor.execute_batch(
            batches
                .into_iter()
                .map(|batch| {
                    let problem = Arc::clone(&problem);
                    move || {
                        let scores = problem.eval_batch(&batch.1);
                        (batch.0, scores, batch.1)
                    }
                })
                .collect(),
        );

        let mut count = 0;
        for (indices, scores, genotypes) in results {
            count += indices.len();
            let score_genotype_iter = scores?.into_iter().zip(genotypes.into_iter());
            for (i, (score, genotype)) in score_genotype_iter.enumerate() {
                let idx = indices[i];
                ecosystem.population[idx].set_score(Some(score));
                ecosystem.population[idx].set_genotype(genotype);
            }
        }

        Ok(count)
    }
}
