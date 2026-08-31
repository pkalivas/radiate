use crate::pipeline::Pipeline;
use crate::{Chromosome, EngineRuntime, Generation, ThreadSync};
use crate::{
    EventHandler,
    message::{
        EcosystemSnapshot, EngineStart, EngineStop, EpochComplete, EpochStart, EventStream,
        Improvement, Subscription,
    },
};
use crate::{GenerationView, builder::GeneticEngineBuilder};
use crate::{context::EvolutionContext, message::Event};
use radiate_core::{Engine, EngineState};
use radiate_core::{EngineStream, error::Result};

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
/// let codec = FloatCodec::matrix(vec![5], 0.0..100.0);
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
    context: EvolutionContext<C, T>,
    pipeline: Pipeline<C>,
    stream: EventStream,
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
    pub(crate) fn new(
        context: EvolutionContext<C, T>,
        pipeline: Pipeline<C>,
        stream: EventStream,
    ) -> Self {
        GeneticEngine {
            context,
            pipeline,
            stream,
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

    /// Returns a clone of the engine's control interface.
    ///
    /// The control interface allows for pausing, resuming, and stopping the engine's execution
    /// from external contexts. If the control interface has not been initialized yet, this method
    /// will create a new instance.
    pub fn control(&mut self) -> ThreadSync {
        self.context.get_or_create_control()
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
    pub fn iter(self) -> EngineRuntime<Self>
    where
        C: 'static,
    {
        EngineRuntime::new(self)
    }

    /// Subscribes to events of type `E` emitted by the engine.
    ///
    /// This method returns a [Subscription] that allows you to define
    /// how to handle events of type `E`. You can use this to listen for events
    /// such as epoch completions, improvements, or custom messages emitted during the evolutionary process.
    pub fn subscribe<E: Event>(&self, handler: impl EventHandler<E>) -> Subscription {
        self.stream.subscribe(handler)
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
    C: Chromosome + Clone + 'static,
    T: Clone + Send + Sync + 'static,
{
    type Epoch = Generation<C, T>;
    type Ctx = EvolutionContext<C, T>;

    fn context(&self) -> &Self::Ctx {
        &self.context
    }

    fn epoch(&self) -> Self::Epoch {
        Generation::from(&self.context)
    }

    fn start(&mut self) {
        self.stream.start();
        self.stream.publish(EngineStart);
    }

    fn stop(&mut self) {
        self.stream.publish(EngineStop::from(&self.context));
    }

    #[inline]
    fn step(&mut self) -> Result<EngineState> {
        if self.context.is_stopped() {
            // We publish a stop event when the `stop` fn is called (above), so
            // no need to publish anything here.
            return Ok(EngineState::Stopped);
        } else if self.context.is_paused() {
            self.stream.publish(EngineState::Paused);
            self.context.wait();
            self.stream.publish(EngineState::Running);
        }

        self.stream.publish(EpochStart::from(&self.context));
        self.pipeline.run(&mut self.context)?;
        if self.context.try_advance_one()? {
            self.stream
                .lazy_publish(|| Improvement::from(&self.context));
        }

        self.stream.publish(EpochComplete::from(&self.context));

        // `Ecosystem` is a heavy clone, but this only clones if we have a subscriber
        // and once it is cloned, the snapshot is backed by an `Arc<Ecosystem>` so
        // we don't pay the clone cost twice.
        self.stream
            .lazy_publish(|| EcosystemSnapshot::from(&self.context));

        Ok(EngineState::Running)
    }
}

/// Implementation of the [EngineStream] trait for [GeneticEngine].
impl<C, T> EngineStream for GeneticEngine<C, T>
where
    C: Chromosome + Clone + 'static,
    T: Clone + Send + Sync,
{
    type View<'a>
        = GenerationView<'a, C, T>
    where
        Self: 'a;

    fn run<F>(mut self, limit: F) -> Result<Self::Epoch>
    where
        F: Fn(&Self::View<'_>) -> bool,
    {
        loop {
            let view = self.step().map(|_| GenerationView::new(&self.context))?;
            if limit(&view) {
                break Ok(self.epoch());
            }
        }
    }
}
