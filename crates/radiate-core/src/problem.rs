//! # Problem Abstraction
//!
//! This module provides the core problem abstraction for genetic algorithms, defining
//! how genotypes are encoded, decoded, and evaluated. The [Problem] trait serves as
//! the central interface that combines encoding/decoding logic with fitness evaluation,
//! making it easy to implement custom problems while maintaining a consistent API.
//!
//! The module provides both the trait definition and concrete implementations:
//! - **Problem trait**: Core interface for genetic algorithm problems
//! - **EngineProblem**: Standard implementation using individual fitness functions
//! - **BatchEngineProblem**: Optimized implementation for batch fitness evaluation

use super::{Chromosome, Codec, Genotype, Score};
use crate::{Objective, error::RadiateResult};
use radiate_error::{RadiateError, radiate_err};
use std::sync::Arc;

/// The core interface for genetic algorithm problems.
///
/// The [Problem] trait encapsulates the three essential components of a genetic
/// algorithm: encoding, decoding, and fitness evaluation. It provides a unified
/// interface that allows the engine to work with any problem implementation
/// without needing to understand the specific details of how genotypes are
/// represented or how fitness is calculated.
///
/// # Generic Parameters
///
/// - `C`: The chromosome type that represents the genetic material
/// - `T`: The phenotype type that represents the decoded individual
///
/// # Thread Safety
///
/// All problems must be `Send + Sync` to support parallel execution across
/// multiple threads. This ensures safe sharing of problems between worker threads
/// during fitness evaluation.
///
/// # Performance Considerations
///
/// - **Encoding**: Called sparingly, so performance is less critical
/// - **Decoding**: Called frequently during evaluation, should be optimized
/// - **Fitness Evaluation**: Debatably the most computationally expensive operation across all problem types
pub trait Problem<C: Chromosome, T>: Send + Sync {
    /// Creates a new [Genotype] representing the initial state of the problem.
    /// The returned [Genotype] should represent a valid starting point for evolution.
    ///
    /// # Returns
    ///
    /// A [Genotype] that can be used as a starting point for evolution
    ///
    /// # Note
    ///
    /// The encoding should produce diverse genotypes to ensure the initial
    /// population has sufficient genetic diversity for effective evolution.
    fn encode(&self) -> Genotype<C>;

    /// Converts a [Genotype] into its corresponding phenotype.
    ///
    /// This method transforms the genetic representation into a form that
    /// can be evaluated by the fitness function. The decoding process should
    /// be deterministic and efficient, as it's called frequently during
    /// fitness evaluation.
    ///
    /// # Arguments
    ///
    /// * [Genotype] - The genotype to decode
    ///
    /// # Returns
    ///
    /// The decoded phenotype that can be evaluated
    ///
    /// # Performance
    ///
    /// This method is called for every individual during fitness evaluation,
    /// so it should be optimized for speed.
    fn decode(&self, genotype: &Genotype<C>) -> T;

    /// Evaluates the fitness of a single individual.
    ///
    /// This method computes the fitness score for a given [Genotype] by first
    /// decoding it to a phenotype and then applying the fitness function.
    /// The fitness score indicates how well the individual solves the problem.
    ///
    /// # Arguments
    ///
    /// * `individual` - The [Genotype] to evaluate
    ///
    /// # Returns
    ///
    /// A fitness score representing the quality of the individual
    fn eval(&self, individual: &Genotype<C>) -> Result<Score, RadiateError>;

    /// Evaluates the fitness of multiple individuals in a batch.
    ///
    /// This method provides an efficient way to evaluate multiple individuals
    /// at once, which can be more efficient than calling `eval` multiple times,
    /// especially when the fitness function can benefit from vectorization,
    /// when there are significant overhead costs per evaluation, or when you need
    /// access to parts, or the whole, of the population to make informed decisions.
    ///
    /// # Arguments
    ///
    /// * `individuals` - A slice of genotypes to evaluate
    ///
    /// # Returns
    ///
    /// A vector of fitness scores, one for each individual
    ///
    /// # Default Implementation
    ///
    /// The default implementation simply calls `eval` for each individual.
    /// Override this method if you can provide a more efficient batch
    /// evaluation strategy.
    ///
    /// # Note
    ///
    /// The order in which the scores are returned must match the order in which
    /// the genotypes are provided.
    fn eval_batch(&self, individuals: &[Genotype<C>]) -> Result<Vec<Score>, RadiateError> {
        individuals.iter().map(|ind| self.eval(ind)).collect()
    }
}

/// [EngineProblem] is a generic, base level concrete implementation of the [Problem] trait that is the
/// default problem used by the engine if none other is specified during building.
///
/// We take the [Codec] and the fitness function from the user and simply wrap them into this struct.
///
/// # Generic Parameters
///
/// - `C`: The [Chromosome] type that represents the genetic material
/// - `T`: The phenotype type that represents the decoded individual
///
/// # Examples
///
/// ```rust
/// use radiate_core::*;
/// use std::sync::Arc;
///
/// // Create a simple fitness function
/// let fitness_fn = Arc::new(|phenotype: Vec<f32>| {
///     Score::from(phenotype.iter().cloned().fold(0.0, f32::max))
/// });
///
/// let problem = EngineProblem {
///     codec: Arc::new(FloatCodec::vector(5, 0.0..1.0)),
///     fitness_fn,
///     objective: Objective::Single(Optimize::Maximize),
/// };
/// ```
///
/// # Thread Safety
///
/// This struct is marked as `Send + Sync` to ensure it can be safely shared
/// across multiple threads during parallel fitness evaluation.
///
/// # Performance Characteristics
///
/// - **Individual Evaluation**: Each individual is evaluated separately
/// - **Memory Usage**: Lower memory overhead per evaluation
/// - **Flexibility**: Easy to implement custom fitness logic
/// - **Parallelization**: Can utilize multiple threads through the executor
pub struct EngineProblem<C, T>
where
    C: Chromosome,
{
    pub objective: Objective,
    pub codec: Arc<dyn Codec<C, T>>,
    pub fitness_fn: Arc<dyn Fn(T) -> Score + Send + Sync>,
}

/// Implementation of [Problem] for [EngineProblem].
///
/// This implementation delegates to the contained codec and fitness function,
/// providing a clean separation of concerns between encoding/decoding logic
/// and fitness evaluation.
impl<C: Chromosome, T> Problem<C, T> for EngineProblem<C, T> {
    fn encode(&self) -> Genotype<C> {
        self.codec.encode()
    }

    fn decode(&self, genotype: &Genotype<C>) -> T {
        self.codec.decode(genotype)
    }

    fn eval(&self, individual: &Genotype<C>) -> RadiateResult<Score> {
        let phenotype = self.decode(individual);
        let score = (self.fitness_fn)(phenotype);

        if self.objective.validate(&score) {
            return Ok(score);
        }

        Err(radiate_err!(
            Evaluation: "Invalid fitness score {:?} for objective {:?}",
            score,
            self.objective
        ))
    }
}

/// Mark [EngineProblem] as thread-safe for parallel execution.
///
/// This implementation is safe because:
/// - `Arc<dyn Codec<C, T>>` is `Send + Sync` when `C` and `T` are `Send + Sync`
/// - `Arc<dyn Fn(T) -> Score + Send + Sync>` is `Send + Sync` by construction
/// - The struct contains no interior mutability
unsafe impl<C: Chromosome, T> Send for EngineProblem<C, T> {}
unsafe impl<C: Chromosome, T> Sync for EngineProblem<C, T> {}

/// A specialized implementation of the [Problem] trait optimized for batch evaluation.
///
/// [BatchEngineProblem] is designed for problems where batch fitness evaluation
/// is significantly more efficient than individual evaluation. It uses a batch
/// fitness function that can process multiple phenotypes at once, potentially
/// leveraging vectorization, shared computations, or parallel processing.
///
/// # Generic Parameters
///
/// - `C`: The [Chromosome] type that represents the genetic material
/// - `T`: The phenotype type that represents the decoded individual
///
/// # Examples
///
/// ```rust
/// use radiate_core::*;
/// use std::sync::Arc;
///
/// // Create a simple fitness function
/// let batch_fitness_fn = Arc::new(|phenotypes: Vec<Vec<f32>>| {
///     phenotypes.iter().map(|p| {
///         Score::from(p.iter().cloned().fold(0.0, f32::max))
///     }).collect()
/// });
///
/// let problem = BatchEngineProblem {
///     codec: Arc::new(FloatCodec::vector(5, 0.0..1.0)),
///     batch_fitness_fn,
///     objective: Objective::Single(Optimize::Maximize),
/// };
/// ```
///
/// # Use Cases
///
/// Use [BatchEngineProblem] when:
/// - Your fitness function can benefit from vectorization
/// - There are shared computations between multiple individuals
/// - The problem domain supports efficient batch processing
/// - You're evaluating large populations where batch overhead is amortized
/// - You need parts or the whole of a population to be evaluated together
pub struct BatchEngineProblem<C, T>
where
    C: Chromosome,
{
    pub objective: Objective,
    pub codec: Arc<dyn Codec<C, T>>,
    pub batch_fitness_fn: Arc<dyn Fn(Vec<T>) -> Vec<Score> + Send + Sync>,
}

/// Implementation of [Problem] for [BatchEngineProblem].
///
/// This implementation provides both individual and batch evaluation methods.
/// The individual evaluation method (`eval`) is implemented by wrapping the
/// phenotype in a single-element slice and calling the batch function, then
/// extracting the first result. This ensures consistency between individual
/// and batch evaluation while maintaining the benefits of batch processing.
impl<C: Chromosome, T> Problem<C, T> for BatchEngineProblem<C, T> {
    fn encode(&self) -> Genotype<C> {
        self.codec.encode()
    }

    fn decode(&self, genotype: &Genotype<C>) -> T {
        self.codec.decode(genotype)
    }

    fn eval(&self, individual: &Genotype<C>) -> RadiateResult<Score> {
        let phenotype = self.decode(individual);
        let scores = (self.batch_fitness_fn)(vec![phenotype]);

        // Cloning a score is a lightweight operation - the internal of a score is a Arc<[f32]>
        // This function will likely never be called anyways as we expect `eval_batch` to be used.
        Ok(scores[0].clone())
    }

    fn eval_batch(&self, individuals: &[Genotype<C>]) -> RadiateResult<Vec<Score>> {
        let phenotypes = individuals
            .iter()
            .map(|genotype| self.decode(genotype))
            .collect::<Vec<T>>();

        let scores = (self.batch_fitness_fn)(phenotypes);

        for i in 0..scores.len() {
            if !self.objective.validate(&scores[i]) {
                return Err(radiate_err!(
                    Evaluation: "Invalid fitness score {:?} for objective {:?}",
                    scores[i],
                    self.objective
                ));
            }
        }

        Ok(scores)
    }
}

/// Mark [BatchEngineProblem] as thread-safe for parallel execution.
///
/// This implementation is safe because:
/// - `Arc<dyn Codec<C, T>>` is `Send + Sync` when `C` and `T` are `Send + Sync`
/// - `Arc<dyn Fn(&[T]) -> Vec<Score> + Send + Sync>` is `Send + Sync` by construction
/// - The struct contains no interior mutability
/// - Batch operations are designed to be thread-safe
unsafe impl<C: Chromosome, T> Send for BatchEngineProblem<C, T> {}
unsafe impl<C: Chromosome, T> Sync for BatchEngineProblem<C, T> {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Chromosome, Codec, FloatChromosome, FloatGene, Gene, Genotype, Optimize, Score};

    #[derive(Debug, Clone)]
    struct MockPhenotype {
        x: f32,
        y: f32,
    }

    struct MockCodec;

    impl Codec<FloatChromosome, MockPhenotype> for MockCodec {
        fn encode(&self) -> Genotype<FloatChromosome> {
            Genotype::new(vec![
                FloatChromosome::from(FloatGene::from(1.0)),
                FloatChromosome::from(FloatGene::from(2.0)),
            ])
        }

        fn decode(&self, genotype: &Genotype<FloatChromosome>) -> MockPhenotype {
            MockPhenotype {
                x: *genotype[0].get(0).allele(),
                y: *genotype[1].get(0).allele(),
            }
        }
    }

    #[test]
    fn test_engine_problem_basic_functionality() {
        let fitness_fn =
            Arc::new(|phenotype: MockPhenotype| Score::from(phenotype.x + phenotype.y));

        let problem = EngineProblem {
            objective: Objective::Single(Optimize::Maximize),
            codec: Arc::new(MockCodec),
            fitness_fn,
        };

        let genotype = problem.encode();
        assert_eq!(genotype.len(), 2);

        let phenotype = problem.decode(&genotype);
        assert_eq!(phenotype.x, 1.0);
        assert_eq!(phenotype.y, 2.0);

        let fitness = problem.eval(&genotype).unwrap();
        assert_eq!(fitness.as_f32(), 3.0);
    }

    #[test]
    fn test_engine_problem_batch_evaluation() {
        let fitness_fn =
            Arc::new(|phenotype: MockPhenotype| Score::from(phenotype.x + phenotype.y));

        let problem = EngineProblem {
            objective: Objective::Single(Optimize::Maximize),
            codec: Arc::new(MockCodec),
            fitness_fn,
        };

        let genotypes = vec![problem.encode(), problem.encode()];

        let scores = problem.eval_batch(&genotypes).unwrap();
        assert_eq!(scores.len(), 2);
        assert_eq!(scores[0].as_f32(), 3.0);
        assert_eq!(scores[1].as_f32(), 3.0);
    }

    #[test]
    fn test_batch_engine_problem_basic_functionality() {
        let batch_fitness_fn = Arc::new(|phenotypes: Vec<MockPhenotype>| {
            phenotypes.iter().map(|p| Score::from(p.x * p.y)).collect()
        });

        let problem = BatchEngineProblem {
            objective: Objective::Single(Optimize::Maximize),
            codec: Arc::new(MockCodec),
            batch_fitness_fn,
        };

        let genotype = problem.encode();
        assert_eq!(genotype.len(), 2);

        let phenotype = problem.decode(&genotype);
        assert_eq!(phenotype.x, 1.0);
        assert_eq!(phenotype.y, 2.0);

        let fitness = problem.eval(&genotype).unwrap();
        assert_eq!(fitness.as_f32(), 2.0); // 1.0 * 2.0
    }

    #[test]
    fn test_batch_engine_problem_batch_evaluation() {
        let batch_fitness_fn = Arc::new(|phenotypes: Vec<MockPhenotype>| {
            phenotypes.iter().map(|p| Score::from(p.x * p.y)).collect()
        });

        let problem = BatchEngineProblem {
            objective: Objective::Single(Optimize::Maximize),
            codec: Arc::new(MockCodec),
            batch_fitness_fn,
        };

        let genotypes = vec![problem.encode(), problem.encode()];

        let scores = problem.eval_batch(&genotypes).unwrap();
        assert_eq!(scores.len(), 2);
        assert_eq!(scores[0].as_f32(), 2.0); // 1.0 * 2.0
        assert_eq!(scores[1].as_f32(), 2.0); // 1.0 * 2.0
    }

    #[test]
    fn test_consistency_between_eval_and_eval_batch() {
        let batch_fitness_fn = Arc::new(|phenotypes: Vec<MockPhenotype>| {
            phenotypes.iter().map(|p| Score::from(p.x * p.y)).collect()
        });

        let problem = BatchEngineProblem {
            objective: Objective::Single(Optimize::Maximize),
            codec: Arc::new(MockCodec),
            batch_fitness_fn,
        };

        let genotype = problem.encode();

        let individual_fitness = problem.eval(&genotype).unwrap();

        let batch_scores = problem.eval_batch(&[genotype.clone()]).unwrap();
        let batch_fitness = &batch_scores[0];

        assert_eq!(individual_fitness.as_f32(), batch_fitness.as_f32());
    }
}
