use crate::Context;
use crate::builder::GeneticEngineBuilder;
use crate::iter::EngineIterator;
use crate::pipeline::Pipeline;
use crate::{Chromosome, EngineEvent};
use crate::{EventBus, Generation};
use radiate_core::{Engine, metric_names};

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
    bus: EventBus<EngineEvent<T>>,
}

impl<C, T> GeneticEngine<C, T>
where
    C: Chromosome + Clone,
    T: Clone + Send + Sync + 'static,
{
    pub(crate) fn new(
        context: Context<C, T>,
        pipeline: Pipeline<C>,
        bus: EventBus<EngineEvent<T>>,
    ) -> Self {
        GeneticEngine {
            context,
            pipeline,
            bus,
        }
    }

    pub fn builder() -> GeneticEngineBuilder<C, T> {
        GeneticEngineBuilder::default()
    }

    pub fn iter(self) -> impl Iterator<Item = Generation<C, T>> {
        EngineIterator { engine: self }
    }
}

impl<C, T> Engine for GeneticEngine<C, T>
where
    C: Chromosome + Clone,
    T: Clone + Send + Sync + 'static,
{
    type Epoch = Generation<C, T>;

    #[inline]
    fn next(&mut self) -> Generation<C, T> {
        if matches!(self.context.index, 0) {
            self.bus.emit(EngineEvent::start());
        }

        self.bus.emit(EngineEvent::epoch_start(&self.context));

        let timer = std::time::Instant::now();
        self.pipeline.run(&mut self.context, &self.bus);
        let elapsed = timer.elapsed();

        self.context
            .epoch_metrics
            .upsert(metric_names::TIME, elapsed);

        self.context.metrics.merge(&self.context.epoch_metrics);

        let best = self.context.ecosystem.population().get(0);
        if let Some(best) = best {
            if let (Some(score), Some(current)) = (best.score(), &self.context.score) {
                if self.context.objective.is_better(score, current) {
                    let score_improvement = current.as_f32() - score.as_f32();
                    self.context
                        .metrics
                        .upsert(metric_names::SCORE_IMPROVEMENT_RATE, score_improvement);

                    self.context.score = Some(score.clone());
                    self.context.best = self.context.problem.decode(best.genotype());
                    self.bus.emit(EngineEvent::improvement(&self.context));
                }
            } else {
                self.context.score = Some(best.score().unwrap().clone());
                self.context.best = self.context.problem.decode(best.genotype());
            }
        }

        self.bus.emit(EngineEvent::epoch_complete(&self.context));

        self.context.index += 1;

        Generation::from(&self.context)
    }
}

impl<C, T> Drop for GeneticEngine<C, T>
where
    C: Chromosome,
    T: Clone + Send + Sync + 'static,
{
    fn drop(&mut self) {
        self.bus.emit(EngineEvent::stop(&self.context));
    }
}
