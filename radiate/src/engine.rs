use super::codexes::Codex;
use super::context::EngineContext;
use super::{GeneticEngineParams, MetricSet, Problem};
use crate::builder::GeneticEngineBuilder;
use crate::domain::timer::Timer;
use crate::{Chromosome, EngineStep};

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
/// use radiate::*;
///
/// // Define a codex that encodes and decodes individuals in the population, in this case using floats.
/// let codex = FloatCodex::matrix(1, 5, 0.0..100.0);
/// // This codex will encode Genotype instances with 1 Chromosome and 5 FloatGenes,
/// // with random alleles between 0.0 and 100.0. It will decode into a Vec<Vec<f32>>.
/// // eg: [[1.0, 2.0, 3.0, 4.0, 5.0]]
///
/// // Create a new instance of the genetic engine with the given codex.
/// let engine = GeneticEngine::from_codex(codex)
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
///         score
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
pub struct GeneticEngine<C, T>
where
    C: Chromosome + 'static,
    T: Clone + Send + 'static,
{
    steps: Vec<Box<dyn EngineStep<C, T>>>,
    params: GeneticEngineParams<C, T>,
}

impl<C, T> GeneticEngine<C, T>
where
    C: Chromosome,
    T: Clone + Send,
{
    /// Create a new instance of the `GeneticEngine` struct with the given parameters.
    /// - `params`: An instance of `GeneticEngineParams` that holds configuration parameters for the genetic engine.
    pub fn new(params: GeneticEngineParams<C, T>, steps: Vec<Box<dyn EngineStep<C, T>>>) -> Self {
        GeneticEngine { params, steps }
    }

    /// Initializes a `GeneticEngineParams` using the provided codex, which defines how individuals
    /// are represented in the population. Because the `Codex` is always needed, this
    /// is a convenience method that allows users to create a `GeneticEngineParams` instance
    /// which will then be 'built' resulting in a `GeneticEngine` instance.
    ///
    /// **Note** with this method, the `Codex` is supplied to the `GeneticEngineParams` and thus
    /// the `GeneticEngineParams` also will need a `FitnessFn` to be supplied before building.
    pub fn from_codex(codex: impl Codex<C, T> + 'static) -> GeneticEngineBuilder<C, T> {
        GeneticEngineBuilder::default().codex(codex)
    }

    /// Initializes a `GeneticEngineParams` using the provided problem, which defines the fitness function
    /// used to evaluate the individuals in the population. Unlike the above method, this method
    /// does not require a `Codex` to be supplied, as the `Problem` will provide the necessary
    /// functionality. So in a sense, the supplying a `Problem` is a method that lets the
    /// user do a little more work up front, but then have less to do when building the `GeneticEngine`.
    ///
    /// Similar to the `from_codex` method, this is a convenience method that allows users
    /// to create a `GeneticEngineParams` instance.
    pub fn from_problem(problem: impl Problem<C, T> + 'static) -> GeneticEngineBuilder<C, T> {
        GeneticEngineBuilder::default().problem(problem)
    }

    /// Executes the genetic algorithm. The algorithm continues until a specified
    /// stopping condition, 'limit', is met, such as reaching a target fitness score or
    /// exceeding a maximum number of generations. When 'limit' returns true, the algorithm stops.
    pub fn run<F>(&self, limit: F) -> EngineContext<C, T>
    where
        F: Fn(&EngineContext<C, T>) -> bool,
    {
        let mut ctx = self.start();

        loop {
            self.next(&mut ctx);

            if limit(&ctx) {
                break self.stop(&mut ctx);
            }
        }
    }

    pub fn next(&self, mut ctx: &mut EngineContext<C, T>) {
        for step in self.steps.iter() {
            step.execute(&mut ctx);
        }
    }

    pub(super) fn start(&self) -> EngineContext<C, T> {
        let population = self.params.population().clone();

        EngineContext {
            population: population.clone(),
            best: self.params.problem().decode(&population[0].genotype()),
            index: 0,
            timer: Timer::new(),
            metrics: MetricSet::new(),
            score: None,
            front: self.params.front().clone(),
            species: Vec::new(),
        }
    }

    fn stop(&self, output: &mut EngineContext<C, T>) -> EngineContext<C, T> {
        output.timer.stop();
        output.clone()
    }
}

impl<C, T> Default for GeneticEngine<C, T>
where
    C: Chromosome,
    T: Clone + Send,
{
    fn default() -> Self {
        GeneticEngineBuilder::<C, T>::default().build()
    }
}

#[cfg(test)]
mod engine_tests {
    use crate::{FloatCodex, GeneticEngine, IntCodex, genetic_test};

    #[test]
    fn engine_can_minimize() {
        let codex = IntCodex::vector(5, 0..100);

        let engine = GeneticEngine::from_codex(codex)
            .minimizing()
            .fitness_fn(|geno: Vec<i32>| geno.iter().sum::<i32>())
            .build();

        let result = engine.run(|ctx| ctx.score().as_i32() == 0);

        let best = result.best;
        assert_eq!(best.iter().sum::<i32>(), 0);
    }

    #[test]
    fn engine_can_maximize() {
        let codex = IntCodex::vector(5, 0..101);

        let engine = GeneticEngine::from_codex(codex)
            .fitness_fn(|geno: Vec<i32>| geno.iter().sum::<i32>())
            .build();

        let result = engine.run(|ctx| ctx.score().as_i32() == 500);

        assert_eq!(result.best.iter().sum::<i32>(), 500);
    }

    #[test]
    fn engine_evolves_towards_target() {
        let target = [1, 2, 3, 4, 5];
        let codex = IntCodex::vector(target.len(), 0..10);

        let engine = GeneticEngine::from_codex(codex)
            .minimizing()
            .fitness_fn(move |geno: Vec<i32>| {
                let mut score = 0;
                for i in 0..geno.len() {
                    score += (geno[i] - target[i]).abs();
                }
                score
            })
            .build();

        let result = engine.run(|ctx| ctx.score().as_i32() == 0);

        assert_eq!(&result.best, &vec![1, 2, 3, 4, 5]);
    }

    genetic_test!(
        name: evolve_zero_vector,
        codex: FloatCodex::vector(5, -10.0..10.0),
        fitness: |geno| geno.iter().map(|x| x * x).sum::<f32>(),
        settings: {
            minimizing,
            population_size: 50,
            num_threads: 4,
        },
        stopping_criteria: |ctx| {
            // Stop when the score is close to zero
            ctx.score().as_f32() < 0.01
        },
        assert: |result| {
            assert!(result.score().as_f32() < 0.1);
        }
    );
}

// self.evaluate(&mut ctx);
// self.speciate(&mut ctx);

// let survivors = self.select_survivors(&mut ctx);
// let offspring = self.create_offspring(&mut ctx);

// self.recombine(&mut ctx, survivors, offspring);

// self.filter(&mut ctx);
// self.evaluate(&mut ctx);
// self.update_front(&mut ctx);
// self.audit(&mut ctx);
