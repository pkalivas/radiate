use crate::Generation;
use crate::builder::GeneticEngineBuilder;
use crate::{Chromosome, EngineIterator, Pipeline};
use radiate_core::engine::Context;
use radiate_core::{Engine, Epoch, metric_names};

/// The `GeneticEngine` is the core component of the Radiate library's genetic algorithm implementation.
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
/// // Define a codex that encodes and decodes individuals in the population, in this case using floats.
/// let codex = FloatCodex::matrix(1, 5, 0.0..100.0);
/// // This codex will encode Genotype instances with 1 Chromosome and 5 FloatGenes,
/// // with random alleles between 0.0 and 100.0. It will decode into a Vec<Vec<f32>>.
/// // eg: [[1.0, 2.0, 3.0, 4.0, 5.0]]
///
/// // Create a new instance of the genetic engine with the given codex.
/// let mut engine = GeneticEngine::builder()
///     .codex(codex) // Set the codex to be used for encoding and decoding individuals.
///     .minimizing()  // Minimize the fitness function.
///     .population_size(150) // Set the population size to 150 individuals.
///     .max_age(15) // Set the maximum age of an individual to 15 generations before it is replaced with a new individual.
///     .offspring_fraction(0.5) // Set the fraction of the population that will be replaced by offspring each generation.
///     .num_threads(4) // Set the number of threads to use in the thread pool for parallel fitness evaluation.
///     .offspring_selector(BoltzmannSelector::new(4_f32)) // Use boltzmann selection to select offspring.
///     .survivor_selector(TournamentSelector::new(3)) // Use tournament selection to select survivors.
///     .alter(alters![
///         ArithmeticMutator::new(0.01), // Specific mutator for numeric values.
///         MeanCrossover::new(0.5) // Specific crossover operation for numeric values.
///     ])
///     .fitness_fn(|genotype: Vec<Vec<f32>>| { // Define the fitness function to be minimized.
///         // Calculate the fitness score of the individual based on the decoded genotype.
///         let score = genotype.iter().fold(0.0, |acc, chromosome| {
///             acc + chromosome.iter().sum::<f32>()
///         });
///         Score::from_f32(score)
///    })
///   .build(); // Build the genetic engine.
///
/// // Run the genetic algorithm until the score of the best individual is 0, then return the result.
/// let result = engine.run(|output| output.score().as_i32() == 0);
/// ```
///
/// # Type Parameters
/// - `C`: The type of the chromosome used in the genotype, which must implement the `Chromosome` trait.
/// - `T`: The type of the phenotype produced by the genetic algorithm, which must be `Clone`, `Send`, and `static`.
pub struct GeneticEngine<C, T, E = Generation<C, T>>
where
    C: Chromosome,
    E: Epoch,
{
    context: Context<C, T>,
    pipeline: Pipeline<C>,
    _epoch: std::marker::PhantomData<E>,
}

impl<C, T, E> GeneticEngine<C, T, E>
where
    C: Chromosome,
    T: Clone + Send,
    E: Epoch,
{
    pub fn new(context: Context<C, T>, pipeline: Pipeline<C>) -> Self {
        GeneticEngine {
            context,
            pipeline,
            _epoch: std::marker::PhantomData,
        }
    }

    pub fn iter(self) -> EngineIterator<C, T, E> {
        EngineIterator { engine: self }
    }
}

impl<C, T> GeneticEngine<C, T, Generation<C, T>>
where
    C: Chromosome,
    T: Clone + Send,
{
    pub fn builder() -> GeneticEngineBuilder<C, T> {
        GeneticEngineBuilder::default()
    }
}

impl<C, T, E> Engine for GeneticEngine<C, T, E>
where
    C: Chromosome,
    E: Epoch<Chromosome = C> + for<'a> From<&'a Context<C, T>>,
{
    type Chromosome = C;
    type Epoch = E;

    fn next(&mut self) -> Self::Epoch {
        let timer = std::time::Instant::now();
        self.pipeline.run(
            self.context.index,
            &mut self.context.metrics,
            &mut self.context.ecosystem,
        );

        self.context
            .metrics
            .upsert_time(metric_names::EVOLUTION_TIME, timer.elapsed());

        let best = self.context.ecosystem.population().get(0);
        if let Some(best) = best {
            if let (Some(score), Some(current)) = (best.score(), &self.context.score) {
                if self.context.objective.is_better(score, current) {
                    self.context.score = Some(score.clone());
                    self.context.best = self.context.problem.decode(best.genotype());
                }
            } else {
                self.context.score = Some(best.score().unwrap().clone());
                self.context.best = self.context.problem.decode(best.genotype());
            }
        }

        self.context.index += 1;

        E::from(&self.context)
    }
}
