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

#[derive(Clone)]
pub enum FitnessKind<C, T>
where
    C: Chromosome,
{
    Single(Arc<dyn Fn(T) -> Score + Send + Sync>),
    Batch(Arc<dyn Fn(Vec<T>) -> Vec<Score> + Send + Sync>),
    RawSingle(Arc<dyn Fn(&Genotype<C>) -> Score + Send + Sync>),
    RawBatch(Arc<dyn Fn(Vec<&Genotype<C>>) -> Vec<Score> + Send + Sync>),
}

impl<C, T> FitnessKind<C, T>
where
    C: Chromosome,
{
    pub fn is_single(&self) -> bool {
        matches!(self, FitnessKind::Single(_) | FitnessKind::RawSingle(_))
    }

    pub fn is_batch(&self) -> bool {
        matches!(self, FitnessKind::Batch(_) | FitnessKind::RawBatch(_))
    }
}

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

/// A standard implementation of the [Problem] trait for general use.
/// [EngineProblem] is a generic, base level concrete implementation of the [Problem] trait that is the
/// default problem used by the engine if none other is specified during building.
///
/// We take the [Codec] and the fitness function from the user and simply wrap them into this struct.
///
///
/// # Generic Parameters
///
/// - `C`: The [Chromosome] type that represents the genetic material
/// - `T`: The phenotype type that represents the decoded individual
pub struct EngineProblem<C, T>
where
    C: Chromosome,
{
    pub objective: Objective,
    pub codec: Arc<dyn Codec<C, T>>,
    pub fitness_kind: FitnessKind<C, T>,
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
        let score = match &self.fitness_kind {
            FitnessKind::RawSingle(raw_fn) => raw_fn(individual),
            FitnessKind::Single(fitness_fn) => {
                let phenotype = self.decode(individual);
                fitness_fn(phenotype)
            }
            _ => {
                return Err(radiate_err!(
                    Fitness: "No suitable fitness function defined for EngineProblem"
                ));
            }
        };

        if self.objective.validate(&score) {
            return Ok(score);
        }

        Err(radiate_err!(
            Fitness: "Invalid fitness score {:?} for objective {:?}",
            score,
            self.objective
        ))
    }

    fn eval_batch(&self, individuals: &[Genotype<C>]) -> RadiateResult<Vec<Score>> {
        let scores = match &self.fitness_kind {
            FitnessKind::RawBatch(raw_batch_fn) => raw_batch_fn(individuals.iter().collect()),
            FitnessKind::Batch(batch_fn) => {
                let phenotypes = individuals
                    .iter()
                    .map(|genotype| self.decode(genotype))
                    .collect::<Vec<T>>();
                batch_fn(phenotypes)
            }
            _ => {
                return Err(radiate_err!(
                    Fitness: "No batch fitness function defined for BatchEngineProblem"
                ));
            }
        };

        let num_individuals = individuals.len();
        let num_scores = scores.len();
        if num_scores != num_individuals {
            return Err(radiate_err!(
                Fitness: "Mismatch between number of individuals and number of scores: expected {}, got {}",
                num_individuals,
                num_scores
            ));
        }

        for value in scores.iter() {
            if !self.objective.validate(value) {
                return Err(radiate_err!(
                    Fitness: "Invalid fitness score {:?} for objective {:?}",
                    value,
                    self.objective
                ));
            }
        }

        Ok(scores)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Chromosome, Codec, FloatChromosome, FloatGene, Gene, Genotype, Score};

    #[derive(Debug, Clone)]
    struct MockPhenotype {
        x: f32,
        y: f32,
    }

    struct MockCodec;

    impl Codec<FloatChromosome<f32>, MockPhenotype> for MockCodec {
        fn encode(&self) -> Genotype<FloatChromosome<f32>> {
            Genotype::new(vec![
                FloatChromosome::from(FloatGene::from(1.0)),
                FloatChromosome::from(FloatGene::from(2.0)),
            ])
        }

        fn decode(&self, genotype: &Genotype<FloatChromosome<f32>>) -> MockPhenotype {
            MockPhenotype {
                x: *genotype[0].get(0).unwrap().allele(),
                y: *genotype[1].get(0).unwrap().allele(),
            }
        }
    }

    #[test]
    fn test_engine_problem_basic_functionality() {
        let fitness_fn =
            Arc::new(|phenotype: MockPhenotype| Score::from(phenotype.x + phenotype.y));

        let problem = EngineProblem {
            objective: Objective::default(),
            codec: Arc::new(MockCodec),
            fitness_kind: FitnessKind::Single(fitness_fn),
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
        let fitness_fn = Arc::new(|phenotype: Vec<MockPhenotype>| {
            phenotype.iter().map(|p| Score::from(p.x + p.y)).collect()
        });

        let problem = EngineProblem {
            objective: Objective::default(),
            codec: Arc::new(MockCodec),
            fitness_kind: FitnessKind::Batch(fitness_fn),
        };

        let genotypes = vec![problem.encode(), problem.encode()];

        let scores = problem.eval_batch(&genotypes).unwrap();
        assert_eq!(scores.len(), 2);
        assert_eq!(scores[0].as_f32(), 3.0);
        assert_eq!(scores[1].as_f32(), 3.0);
    }
}
