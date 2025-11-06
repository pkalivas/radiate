use crate::Chromosome;
use crate::builder::GeneticEngineBuilder;
use crate::context::Context;
use crate::events::EngineMessage;
use crate::iter::EngineIterator;
use crate::pipeline::Pipeline;
use crate::{EventBus, Generation};
use radiate_core::Engine;
use radiate_core::error::Result;

/// The [GeneticEngine] is the core component of the Radiate library's genetic algorithm implementation.
/// The engine is designed to be fast, flexible and extensible, allowing users to
/// customize various aspects of the genetic algorithm to suit their specific needs.
///
/// Essentially, it is a high-level abstraction that orchestrates all aspects of the genetic algorithm. It is
/// responsible for managing the population of individuals, evaluating the fitness of each individual,
/// selecting the individuals that will survive to the next generation, and creating the next generation through
/// crossover and mutation.
///
/// # Examples
/// ``` no_run
/// use radiate_engines::*;
///
/// // Define a codec that encodes and decodes individuals in the population, in this case using floats.
/// let codec = FloatCodec::matrix(1, 5, 0.0..100.0);
/// // This codec will encode Genotype instances with 1 Chromosome and 5 FloatGenes,
/// // with random alleles between 0.0 and 100.0. It will decode into a Vec<Vec<f32>>.
/// // eg: [[1.0, 2.0, 3.0, 4.0, 5.0]]
///
/// // Create a new instance of the genetic engine with the given codec.
/// let mut engine = GeneticEngine::builder()
///     .codec(codec)
///     .minimizing()
///     .population_size(150)
///     .max_age(15)
///     .offspring_fraction(0.5)
///     .offspring_selector(BoltzmannSelector::new(4_f32))
///     .survivor_selector(TournamentSelector::new(3))
///     .alter(alters![
///         ArithmeticMutator::new(0.01),
///         MeanCrossover::new(0.5)
///     ])
///     .fitness_fn(|genotype: Vec<Vec<f32>>| {
///         genotype.iter().fold(0.0, |acc, chromosome| {
///             acc + chromosome.iter().sum::<f32>()
///         })
///    })
///   .build();
///
/// // Run the genetic algorithm until the score of the best individual is 0, then return the result.
/// let result = engine.run(|output| output.score().as_i32() == 0);
/// ```
///
/// # Type Parameters
/// - `C`: The type of the chromosome used in the genotype, which must implement the [Chromosome] trait.
/// - `T`: The type of the phenotype produced by the genetic algorithm, which must be `Clone`, `Send`, and `static`.
pub struct GeneticEngine<C, T>
where
    C: Chromosome,
    T: Clone + Send + Sync + 'static,
{
    context: Context<C, T>,
    pipeline: Pipeline<C>,
    bus: EventBus<T>,
}

impl<C, T> GeneticEngine<C, T>
where
    C: Chromosome + Clone,
    T: Clone + Send + Sync + 'static,
{
    /// Creates a new genetic engine with the specified components.
    ///
    /// This constructor is primarily used internally by the builder pattern.
    /// Users should create engines using `GeneticEngine::builder()`.
    pub(crate) fn new(context: Context<C, T>, pipeline: Pipeline<C>, bus: EventBus<T>) -> Self {
        GeneticEngine {
            context,
            pipeline,
            bus,
        }
    }

    /// Creates a new builder for configuring and constructing a genetic engine.
    ///
    /// The builder pattern provides a fluent interface for configuring all aspects
    /// of the genetic algorithm, including population settings, selection strategies,
    /// evolutionary operators, and fitness functions.
    pub fn builder() -> GeneticEngineBuilder<C, T> {
        GeneticEngineBuilder::default()
    }

    /// Converts the engine into an iterator that yields generations.
    ///
    /// This method allows you to iterate over the evolutionary process manually,
    /// giving you fine-grained control over when and how generations are processed.
    /// The iterator yields `Generation` objects containing the current state and
    /// statistics for each generation.
    ///
    /// # Use Cases
    ///
    /// Manual iteration is useful when you need to:
    /// - Implement custom termination logic
    /// - Monitor progress between generations
    /// - Apply external control or adaptation
    /// - Integrate with custom monitoring systems
    /// - Implement interactive evolutionary algorithms
    ///
    /// # Note
    ///
    /// The iterator consumes the engine, so you can only iterate once. If you need
    /// to run the engine multiple times, create a new instance using the builder.
    pub fn iter(self) -> impl Iterator<Item = Generation<C, T>> {
        EngineIterator { engine: self }
    }
}

/// Implementation of the [Engine] trait for [GeneticEngine].
///
/// This implementation provides the core evolutionary logic, advancing the
/// population through one complete generation cycle. Each call to `next()`
/// represents one generation of evolution, including fitness evaluation,
/// selection, reproduction, and population replacement.
///
/// # Evolutionary Cycle
///
/// Each generation follows this sequence:
/// 1. **Event Emission**: Start of epoch events
/// 2. **Pipeline Execution**: Run evolutionary operators
/// 3. **Metrics Collection**: Record timing and performance data
/// 4. **Best Individual Update**: Track improvements and best solutions
/// 5. **Event Completion**: End of epoch events
/// 6. **Generation Advancement**: Increment generation counter
///
/// # Performance Optimizations
///
/// - **Efficient Metrics**: Metrics are updated incrementally to minimize overhead
/// - **Event Batching**: Events are emitted efficiently without blocking execution
/// - **Pipeline Optimization**: Evolutionary operators are executed in optimized sequences
impl<C, T> Engine for GeneticEngine<C, T>
where
    C: Chromosome + Clone,
    T: Clone + Send + Sync + 'static,
{
    type Epoch = Generation<C, T>;

    #[inline]
    fn next(&mut self) -> Result<Generation<C, T>> {
        if matches!(self.context.index, 0) {
            self.bus.publish(EngineMessage::<C, T>::Start);
        }

        self.bus.publish(EngineMessage::EpochStart(&self.context));
        self.pipeline.run(&mut self.context)?;
        self.bus.publish(EngineMessage::EpochEnd(&self.context));

        if self.context.try_advance_one() {
            self.bus.publish(EngineMessage::Improvement(&self.context));
        }

        Ok(Generation::from(&self.context))
    }
}

/// Custom drop implementation for proper cleanup and event emission.
///
/// When the engine is dropped, it emits a stop event to notify any listeners
/// that the evolutionary process has ended. This allows external systems to
/// perform cleanup operations or finalize results.
///
/// # Event Emission
///
/// The stop event includes the final context state, allowing listeners to:
/// - Record final metrics and statistics
/// - Save final population state
/// - Perform cleanup operations
/// - Generate final reports
/// - Integrate with external systems
impl<C, T> Drop for GeneticEngine<C, T>
where
    C: Chromosome,
    T: Clone + Send + Sync + 'static,
{
    fn drop(&mut self) {
        self.bus.publish(EngineMessage::Stop(&self.context));
    }
}
