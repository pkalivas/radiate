use std::collections::HashSet;
use std::sync::Arc;

use crate::engines::alterers::alter::Alter;
use crate::engines::genetic_engine_params::GeneticEngineParams;
use crate::engines::genome::genes::gene::Gene;
use crate::engines::genome::population::Population;
use crate::engines::optimize::Optimize;
use crate::engines::schema::timer::Timer;
use crate::engines::score::Score;

use super::codexes::Codex;
use super::engine_context::EngineContext;
use super::genome::phenotype::Phenotype;
use super::selectors::select::Select;
use super::{
    MetricSet, ThreadPool, METRIC_AGE, METRIC_AGE_FILTER, METRIC_EVALUATE, METRIC_INVALID_FILTER,
    METRIC_SCORE, METRIC_UNIQUE,
};

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
/// ``` rust
/// use radiate::*;
///
/// // Define a codex that encodes and decodes individuals in the population, in this case using floats.
/// let codex = FloatCodex::new(1, 5, 0.0, 100.0);
/// // This codex will encode Genotype instances with 1 Chromosome and 5 FloatGenes,
/// // with random allels between 0.0 and 100.0. It will decode into a Vec<Vec<f32>>.
/// // eg: [[1.0, 2.0, 3.0, 4.0, 5.0]]
///
/// // Create a new instance of the genetic engine with the given codex.
/// let engine = GeneticEngine::from_codex(&codex)
///     .minimizing()  // Minimize the fitness function.
///     .population_size(150) // Set the population size to 150 individuals.
///     .max_age(15) // Set the maximum age of an individual to 15 generations before it is replaced with a new individual.
///     .offspring_fraction(0.5) // Set the fraction of the population that will be replaced by offspring each generation.
///     .num_threads(4) // Set the number of threads to use in the thread pool for parallel fitness evaluation.
///     .offspring_selector(BoltzmannSelector::new(4_f32)) // Use boltzmann selection to select offspring.
///     .survivor_selector(TournamentSelector::new(3)) // Use tournament selection to select survivors.
///     .alterer(vec![                                      
///         Alterer::mutation(NumericMutator::new(0.01)), // Specific mutator for numeric values.
///         Alterer::crossover(MeanCrossover::new(0.5)) // Specific crossover operation for numeric values.
///     ])
///     .fitness_fn(|genotype: Vec<Vec<f32>>| { // Define the fitness function to be minimized.
///         // Calculate the fitness score of the individual based on the decoded genotype.
///         let score = genotype.iter().fold(0.0, |acc, chromosome| {
///             acc + chromosome.iter().fold(0.0, |acc, gene| acc + gene)
///         });
///         Score::from_f32(score)
///    })
///   .build(); // Build the genetic engine.
///
/// // Run the genetic algorithm until the score of the best individual is 0, then return the result.
/// let result = engine.run(|output| output.score().as_int() == 0);
/// ```
///
/// # Type Parameters
/// - `G`: The type of gene used in the genetic algorithm, which must implement the `Gene` trait.
/// - `A`: The type of the allele associated with the gene - the gene's "expression".
/// - `T`: The type of the phenotype produced by the genetic algorithm, which must be `Clone`, `Send`, and `static`.
///
pub struct GeneticEngine<'a, G, A, T>
where
    G: Gene<G, A>,
    T: Clone + Send + 'static,
{
    pub params: GeneticEngineParams<'a, G, A, T>,
}

impl<'a, G, A, T> GeneticEngine<'a, G, A, T>
where
    G: Gene<G, A>,
    T: Clone + Send,
{
    /// Create a new instance of the `GeneticEngine` struct with the given parameters.
    /// - `params`: An instance of `GeneticEngineParams` that holds configuration parameters for the genetic engine.
    pub fn new(params: GeneticEngineParams<'a, G, A, T>) -> Self {
        GeneticEngine { params }
    }

    /// Initializes a `GeneticEngineParams` using the provided codex, which defines how individuals
    /// are represented in the population. Because the `Codex` is always needed, this
    /// is a convenience method that allows users to create a `GeneticEngineParams` instance
    /// which will then be 'built' resulting in a `GeneticEngine` instance.
    pub fn from_codex(codex: &'a impl Codex<G, A, T>) -> GeneticEngineParams<G, A, T> {
        GeneticEngineParams::new().codex(codex)
    }

    /// Executes the genetic algorithm. The algorithm continues until a specified
    /// stopping condition, 'limit', is met, such as reaching a target fitness score or
    /// exceeding a maximum number of generations. When 'limit' returns true, the algorithm stops.
    pub fn run<F>(&self, limit: F) -> EngineContext<G, A, T>
    where
        F: Fn(&EngineContext<G, A, T>) -> bool,
    {
        let mut ctx = self.start();

        loop {
            self.evaluate(&mut ctx);

            let mut survivors = self.select_survivors(&ctx.population, &mut ctx.metrics);
            let mut offspring = self.select_offspring(&ctx.population, &mut ctx.metrics);

            self.alter(&mut offspring, &mut ctx.metrics, ctx.index);

            self.filter(&mut survivors, &mut ctx.metrics, ctx.index);
            self.filter(&mut offspring, &mut ctx.metrics, ctx.index);

            self.recombine(&mut ctx, survivors, offspring);

            self.evaluate(&mut ctx);
            self.audit(&mut ctx);

            if limit(&ctx) {
                break self.stop(&mut ctx);
            }
        }
    }

    /// Evaluates the fitness of each individual in the population using the fitness function
    /// provided in the genetic engine parameters. The fitness function is a closure that takes
    /// a phenotype as input and returns a score. The score is then used to rank the individuals
    /// in the population.
    ///
    /// Importantly, this method uses a thread pool to evaluate the fitness of each individual in
    /// parallel, which can significantly speed up the evaluation process for large populations.
    /// It will also only evaluate individuals that have not yet been scored, which saves time
    /// by avoiding redundant evaluations.
    fn evaluate(&self, handle: &mut EngineContext<G, A, T>) {
        let codex = self.codex();
        let optimize = self.optimize();
        let thread_pool = self.thread_pool();
        let timer = Timer::new();

        let mut work_results = Vec::new();
        for idx in 0..handle.population.len() {
            let individual = handle.population.get(idx);
            if !individual.score().is_some() {
                let fitness_fn = self.fitness_fn();
                let decoded = codex.decode(individual.genotype());
                let work = thread_pool.task(move || (idx, fitness_fn(decoded)));

                work_results.push(work);
            }
        }

        let count = work_results.len() as f32;
        for work_result in work_results {
            let (idx, score) = work_result.result();
            handle.population.get_mut(idx).set_score(Some(score));
        }

        handle.upsert_metric(METRIC_EVALUATE, count, Some(timer.duration()));

        optimize.sort(&mut handle.population);
    }

    /// Selects the individuals that will survive to the next generation. The number of survivors
    /// is determined by the population size and the offspring fraction specified in the genetic
    /// engine parameters. The survivors are selected using the survivor selector specified in the
    /// genetic engine parameters, which is typically a selection algorithm like tournament selection
    /// or roulette wheel selection. For example, if the population size is 100 and the offspring
    /// fraction is 0.8, then 20 individuals will be selected as survivors.
    ///
    /// This method returns a new population containing only the selected survivors.
    fn select_survivors(
        &self,
        population: &Population<G, A>,
        metrics: &mut MetricSet,
    ) -> Population<G, A> {
        let selector = self.survivor_selector();
        let count = self.survivor_count();
        let optimize = self.optimize();

        let timer = Timer::new();
        let result = selector.select(population, optimize, count);

        metrics.upsert(selector.name(), count as f32, timer.duration());

        result
    }

    /// Selects the offspring that will be used to create the next generation. The number of offspring
    /// is determined by the population size and the offspring fraction specified in the genetic
    /// engine parameters. The offspring are selected using the offspring selector specified in the
    /// genetic engine parameters, which, like the survivor selector, is typically a selection algorithm
    /// like tournament selection or roulette wheel selection. For example, if the population size is 100
    /// and the offspring fraction is 0.8, then 80 individuals will be selected as offspring which will
    /// be used to create the next generation through crossover and mutation.
    ///
    /// This method returns a new population containing only the selected offspring.
    fn select_offspring(
        &self,
        population: &Population<G, A>,
        metrics: &mut MetricSet,
    ) -> Population<G, A> {
        let selector = self.offspring_selector();
        let count = self.offspring_count();
        let optimize = self.optimize();

        let timer = Timer::new();
        let result = selector.select(population, optimize, count);

        metrics.upsert(selector.name(), count as f32, timer.duration());

        result
    }

    /// Alters the offspring population using the alterers specified in the genetic engine parameters.
    /// The alterer in this case is going to be a ```CompositeAlterer``` and is responsible for applying
    /// the provided mutation and crossover operations to the offspring population.
    fn alter(&self, population: &mut Population<G, A>, metrics: &mut MetricSet, generation: i32) {
        let alterer = self.alterer();
        let optimize = self.optimize();

        let alter_metrics = alterer.alter(population, optimize, generation);
        for metric in alter_metrics {
            metrics.upsert_metric(metric);
        }
    }

    /// Filters the population to remove individuals that are too old or invalid. The maximum age
    /// of an individual is determined by the 'max_age' parameter in the genetic engine parameters.
    /// If an individual's age exceeds this limit, it is replaced with a new individual. Similarly,
    /// if an individual is found to be invalid (i.e., its genotype is not valid, provided by the ```Valid``` trait),
    /// it is replaced with a new individual. This method ensures that the population remains
    /// healthy and that only valid individuals are allowed to reproduce or survive to the next generation.
    fn filter(&self, population: &mut Population<G, A>, metrics: &mut MetricSet, generation: i32) {
        let max_age = self.params.max_age;
        let codex = self.codex();

        let timer = Timer::new();
        let mut age_count = 0;
        let mut invalid_count = 0;
        for i in 0..population.len() {
            let phenotype = population.get(i);

            if phenotype.age(generation) > max_age {
                population.set(i, Phenotype::from_genotype(codex.encode(), generation));
                age_count += 1;
            } else if !phenotype.genotype().is_valid() {
                population.set(i, Phenotype::from_genotype(codex.encode(), generation));
                invalid_count += 1;
            }
        }

        metrics.upsert(METRIC_AGE_FILTER, age_count as f32, timer.duration());
        metrics.upsert(
            METRIC_INVALID_FILTER,
            invalid_count as f32,
            timer.duration(),
        );
    }

    /// Recombines the survivors and offspring populations to create the next generation. The survivors
    /// are the individuals from the previous generation that will survive to the next generation, while the
    /// offspring are the individuals that were selected from the previous generation then altered.
    /// This method combines the survivors and offspring populations into a single population that
    /// will be used in the next iteration of the genetic algorithm.
    fn recombine(
        &self,
        handle: &mut EngineContext<G, A, T>,
        survivors: Population<G, A>,
        offspring: Population<G, A>,
    ) {
        handle.population = survivors
            .into_iter()
            .chain(offspring)
            .collect::<Population<G, A>>();
    }

    /// Audits the current state of the genetic algorithm, updating the best individual found so far
    /// and calculating various metrics such as the age of individuals, the score of individuals, and the
    /// number of unique scores in the population. This method is called at the end of each generation.
    fn audit(&self, output: &mut EngineContext<G, A, T>) {
        let codex = self.codex();
        let optimize = self.optimize();

        if !output.population.is_sorted {
            self.optimize().sort(&mut output.population);
        }

        if let Some(current_score) = &output.score {
            if let Some(best_score) = output.population.get(0).score() {
                if optimize.is_better(best_score, current_score) {
                    output.score = Some(best_score.clone());
                    output.best = codex.decode(output.population.get(0).genotype());
                }
            }
        } else {
            output.score = output.population.get(0).score().clone();
            output.best = codex.decode(output.population.get(0).genotype());
        }

        self.add_metrics(output);

        output.index += 1;
    }

    /// Adds various metrics to the output context, including the age of individuals, the score of individuals,
    /// and the number of unique scores in the population. These metrics can be used to monitor the progress of
    /// the genetic algorithm and to identify potential issues or areas for improvement.
    ///
    /// The age of an individual is the number of generations it has survived, while the score of an individual
    /// is a measure of its fitness. The number of unique scores in the population is a measure of diversity, with
    /// a higher number indicating a more diverse population.
    fn add_metrics(&self, output: &mut EngineContext<G, A, T>) {
        let mut unique = HashSet::new();
        for i in 0..output.population.len() {
            let phenotype = output.population.get(i);

            let age = phenotype.age(output.index);
            let score = phenotype.score().as_ref().unwrap();

            output.metrics.upsert_value(METRIC_AGE, age as f32);
            output.metrics.upsert_value(METRIC_SCORE, score.as_float());
            unique.insert(score.clone());
        }

        output
            .metrics
            .upsert_value(METRIC_UNIQUE, unique.len() as f32);
    }

    /// Returns the survivor selector specified in the genetic engine parameters. The survivor selector is
    /// responsible for selecting the individuals that will survive to the next generation.
    fn survivor_selector(&self) -> &dyn Select<G, A> {
        self.params.survivor_selector.as_ref()
    }

    /// Returns the offspring selector specified in the genetic engine parameters. The offspring selector is
    /// responsible for selecting the offspring that will be used to create the next generation through crossover
    /// and mutation.
    fn offspring_selector(&self) -> &dyn Select<G, A> {
        self.params.offspring_selector.as_ref()
    }

    /// Returns the alterer specified in the genetic engine parameters. The alterer is responsible for applying
    /// the provided mutation and crossover operations to the offspring population.
    fn alterer(&self) -> &impl Alter<G, A> {
        self.params.alterer.as_ref().unwrap()
    }

    /// Returns the codex specified in the genetic engine parameters. The codex is responsible for encoding and
    /// decoding individuals in the population, converting between the genotype and phenotype representations.
    fn codex(&self) -> &'a dyn Codex<G, A, T> {
        *Arc::clone(self.params.codex.as_ref().unwrap())
    }

    /// Returns the fitness function specified in the genetic engine parameters. The fitness function is a closure
    /// that takes a 'T' (the decoded Genotype) as input and returns a score. The score is used to rank the individuals in the population.
    fn fitness_fn(&self) -> Arc<dyn Fn(T) -> Score + Send + Sync> {
        Arc::clone(self.params.fitness_fn.as_ref().unwrap())
    }

    /// Returns the population specified in the genetic engine parameters. This is only called at the start of the genetic algorithm.
    fn population(&self) -> &Population<G, A> {
        self.params.population.as_ref().unwrap()
    }

    /// Returns the optimize function specified in the genetic engine parameters. The optimize function is responsible
    /// for sorting the population based on the fitness of the individuals. This is typically done in descending order,
    /// with the best individuals at the front of the population.
    fn optimize(&self) -> &Optimize {
        &self.params.optimize
    }

    /// Returns the number of survivors in the population. This is calculated based on the population size and the offspring fraction.
    fn survivor_count(&self) -> usize {
        self.params.population_size - self.offspring_count()
    }

    /// Returns the number of offspring in the population. This is calculated based on the population size and the offspring fraction.
    /// For example, if the population size is 100 and the offspring fraction is 0.8, then 80 individuals will be selected as offspring.
    fn offspring_count(&self) -> usize {
        (self.params.population_size as f32 * self.params.offspring_fraction) as usize
    }

    /// Returns the thread pool specified in the genetic engine parameters. The thread pool is used to evaluate the fitness of
    /// individuals in parallel, which can significantly speed up the evaluation process for large populations.
    fn thread_pool(&self) -> &ThreadPool {
        &self.params.thread_pool
    }

    /// Starts the genetic algorithm by initializing the population and returning the initial state of the genetic engine.
    fn start(&self) -> EngineContext<G, A, T> {
        let population = self.population();

        EngineContext {
            population: population.clone(),
            best: self.codex().decode(population.get(0).genotype()),
            index: 0,
            timer: Timer::new(),
            metrics: MetricSet::new(),
            score: None,
        }
    }

    /// Stops the genetic algorithm by stopping the timer and returning the final state of the genetic engine.
    fn stop(&self, output: &mut EngineContext<G, A, T>) -> EngineContext<G, A, T> {
        output.timer.stop();
        output.clone()
    }
}
