use super::codexes::Codex;
use super::context::EngineContext;
use super::thread_pool::ThreadPool;
use super::{Alter, EngineError, GeneticEngineParams, Problem, ReplacementStrategy};
use crate::engines::builder::GeneticEngineBuilder;
use crate::engines::domain::timer::Timer;
use crate::engines::genome::population::Population;
use crate::engines::objectives::Score;
use crate::objectives::Objective;
use crate::{Chromosome, Metric, Select, Valid, metric_names};
use std::sync::Arc;

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
/// let codex = FloatCodex::new(1, 5, 0.0..100.0);
/// // This codex will encode Genotype instances with 1 Chromosome and 5 FloatGenes,
/// // with random allels between 0.0 and 100.0. It will decode into a Vec<Vec<f32>>.
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
pub struct GeneticEngine<C, T>
where
    C: Chromosome + 'static,
    T: Clone + Default + Send + 'static,
{
    params: GeneticEngineParams<C, T>,
}

impl<C, T> GeneticEngine<C, T>
where
    C: Chromosome,
    T: Clone + Default + Send,
{
    /// Create a new instance of the `GeneticEngine` struct with the given parameters.
    /// - `params`: An instance of `GeneticEngineParams` that holds configuration parameters for the genetic engine.
    pub fn new(params: GeneticEngineParams<C, T>) -> Self {
        GeneticEngine { params }
    }

    pub fn builder() -> GeneticEngineBuilder<C, T> {
        GeneticEngineBuilder::default()
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
            self.evaluate(&mut ctx);

            let survivors = self.select_survivors(&mut ctx);
            let offspring = self.create_offspring(&mut ctx);

            self.recombine(&mut ctx, survivors, offspring);

            self.filter(&mut ctx);
            self.evaluate(&mut ctx);
            self.audit(&mut ctx);

            if ctx.is_err() || limit(&ctx) {
                break self.stop(&mut ctx);
            }
        }
    }

    /// Evaluates the fitness of each individual in the population using the fitness function
    /// provided in the genetic engine parameters. The score is then used to rank the individuals
    /// in the population.
    ///
    /// Importantly, this method uses a thread pool to evaluate the fitness of each individual in
    /// parallel, which can significantly speed up the evaluation process for large populations.
    /// It will also only evaluate individuals that have not yet been scored, which saves time
    /// by avoiding redundant evaluations.
    fn evaluate(&self, handle: &mut EngineContext<C, T>) {
        if handle.is_err() {
            return;
        }

        let objective = self.objective();
        let thread_pool = self.thread_pool();
        let timer = Timer::new();

        let mut work_results = Vec::new();
        for idx in 0..handle.population.len() {
            let individual = &mut handle.population[idx];
            if individual.score().is_some() {
                continue;
            } else {
                let problem = self.problem();
                let geno = individual.take_genotype();
                let work = thread_pool.submit_with_result(move || {
                    let score = problem.eval(&geno);
                    (idx, score, geno)
                });

                work_results.push(work);
            }
        }

        let count = work_results.len() as f32;
        for work_result in work_results {
            let (idx, score, genotype) = work_result.result();
            match score {
                Ok(score) => {
                    handle.population[idx].set_score(Some(score));
                    handle.population[idx].set_genotype(genotype);
                }
                Err(e) => {
                    handle.set_error(e);
                    return;
                }
            }
        }

        handle.upsert_operation(metric_names::EVALUATION, count, timer);

        objective.sort(&mut handle.population);
    }

    /// the `select_survivors` method selects the individuals that will survive
    /// to the next generation. The number of survivors is determined by the population size and the
    /// offspring fraction specified in the genetic engine parameters. The survivors
    /// are selected using the survivor selector specified in the genetic engine parameters,
    /// which is typically a selection algorithm like tournament selection
    /// or roulette wheel selection. For example, if the population size is 100 and the offspring
    /// fraction is 0.8, then 20 individuals will be selected as survivors.
    ///
    /// The reasoning behind this is to ensure that a subset of the population is retained
    /// to the next generation, while the rest of the population is replaced with new individuals created
    /// through crossover and mutation. By selecting a subset of the population to survive, the genetic algorithm
    /// can maintain some of the best solutions found so far while also introducing new genetic material/genetic diversity.
    ///
    /// This method returns a new population containing only the selected survivors.
    fn select_survivors(&self, ctx: &mut EngineContext<C, T>) -> Option<Population<C>> {
        if ctx.is_err() {
            return None;
        }

        let selector = self.survivor_selector();
        let count = self.survivor_count();
        let objective = self.objective();

        let timer = Timer::new();

        selector
            .select(&ctx.population, objective, count)
            .map(|survivors| {
                ctx.upsert_operation(selector.name(), survivors.len() as f32, timer);
                survivors
            })
            .map_err(|selector_error| {
                ctx.set_error(selector_error);
            })
            .ok()
    }

    /// Create the offspring that will be used to create the next generation. The number of offspring
    /// is determined by the population size and the offspring fraction specified in the genetic
    /// engine parameters. The offspring are selected using the offspring selector specified in the
    /// genetic engine parameters, which, like the survivor selector, is typically a selection algorithm
    /// like tournament selection or roulette wheel selection. For example, if the population size is 100
    /// and the offspring fraction is 0.8, then 80 individuals will be selected as offspring which will
    /// be used to create the next generation through crossover and mutation.
    ///
    /// Once the parents are selected through the offspring selector, the `create_offspring` method
    /// will apply the mutation and crossover operations specified during engine creation to the
    /// selected parents, creating a new population of `Phenotypes` with the same size as the
    /// `offspring_fraction` specifies. This process introduces new genetic material into the population,
    /// which allows the genetic algorithm explore new solutions in the problem space and (hopefully)
    /// avoid getting stuck in local minima.
    fn create_offspring(&self, ctx: &mut EngineContext<C, T>) -> Option<Population<C>> {
        if ctx.is_err() {
            return None;
        }

        let selector = self.offspring_selector();
        let count = self.offspring_count();
        let objective = self.objective();
        let alters = self.alters();

        let timer = Timer::new();

        selector
            .select(&ctx.population, objective, count)
            .map(|mut offspring| {
                ctx.upsert_operation(selector.name(), offspring.len() as f32, timer);

                objective.sort(&mut offspring);

                for alterer in alters {
                    let alter_result = alterer.alter(&mut offspring, ctx.index);

                    for metric in alter_result {
                        ctx.metrics.upsert(metric);
                    }
                }

                offspring
            })
            .map_err(|selector_error| {
                ctx.set_error(selector_error);
            })
            .ok()
    }

    /// Filters the population to remove individuals that are too old or invalid. The maximum age
    /// of an individual is determined by the 'max_age' parameter in the genetic engine parameters.
    /// If an individual's age exceeds this limit, it is replaced with a new individual. Similarly,
    /// if an individual is found to be invalid (i.e., its genotype is not valid, provided by the `valid` trait),
    /// it is replaced with a new individual. This method ensures that the population remains
    /// healthy and that only valid individuals are allowed to reproduce or survive to the next generation.
    ///
    /// The method in which a new individual is created is determined by the `filter_strategy`
    /// parameter in the genetic engine parameters and is either `FilterStrategy::Encode` or
    /// `FilterStrategy::PopulationSample`. If the `FilterStrategy` is `FilterStrategy::Encode`, then a new individual
    /// is created using the `encode` method of the `Problem` trait, while if the `FilterStrategy`
    /// is `FilterStrategy::PopulationSample`, then a new individual is created by randomly selecting
    /// an individual from the population.
    fn filter(&self, context: &mut EngineContext<C, T>) {
        if context.is_err() {
            return;
        }

        let max_age = self.max_age();

        let generation = context.index;
        let population = &mut context.population;

        let timer = Timer::new();
        let mut age_count = 0_f32;
        let mut invalid_count = 0_f32;
        for i in 0..population.len() {
            let phenotype = &population[i];

            let mut removed = false;
            if phenotype.age(generation) > max_age {
                removed = true;
                age_count += 1_f32;
            } else if !phenotype.genotype().is_valid() {
                removed = true;
                invalid_count += 1_f32;
            }

            if removed {
                let replacement = self.replace_strategy();
                let problem = self.problem();
                let encoder = Arc::new(move || problem.encode());

                let replace_result = replacement.replace(i, generation, population, encoder);
                if let Err(e) = replace_result {
                    context.set_error(e);
                    return;
                }
            }
        }

        let duration = timer.duration();
        context.upsert_operation(metric_names::AGE_FILTER, age_count, duration);
        context.upsert_operation(metric_names::INVALID_FILTER, invalid_count, duration);
    }

    /// Recombines the survivors and offspring populations to create the next generation. The survivors
    /// are the individuals from the previous generation that will survive to the next generation, while the
    /// offspring are the individuals that were selected from the previous generation then altered.
    /// This method combines the survivors and offspring populations into a single population that
    /// will be used in the next iteration of the genetic algorithm.
    fn recombine(
        &self,
        handle: &mut EngineContext<C, T>,
        survivors: Option<Population<C>>,
        offspring: Option<Population<C>>,
    ) {
        if survivors.is_some() && offspring.is_some() && !handle.is_err() {
            match (survivors, offspring) {
                (Some(survivors), Some(offspring)) => {
                    handle.population = survivors
                        .into_iter()
                        .chain(offspring)
                        .collect::<Population<C>>();
                }
                _ => {}
            }
        }
    }

    /// Audits the current state of the genetic algorithm, updating the best individual found so far
    /// and calculating various metrics such as the age of individuals, the score of individuals, and the
    /// number of unique scores in the population. This method is called at the end of each generation.
    fn audit(&self, output: &mut EngineContext<C, T>) {
        if output.is_err() {
            return;
        }

        let problem = self.problem();
        let optimize = self.objective();

        if !output.population.is_sorted {
            optimize.sort(&mut output.population);
        }

        if let Some(current_score) = &output.score {
            if let Some(best_score) = output.population[0].score() {
                if optimize.is_better(best_score, current_score) {
                    output.score = Some(best_score.clone());
                    match problem.decode(output.population[0].genotype()) {
                        Ok(best) => output.best = best,
                        Err(e) => {
                            output.set_error(e);
                            return;
                        }
                    }
                }
            }
        } else {
            output.score = output.population[0].score().cloned();
            match problem.decode(output.population[0].genotype()) {
                Ok(best) => output.best = best,
                Err(e) => {
                    output.set_error(e);
                    return;
                }
            }
        }

        self.update_front(output);
        self.update_metrics(output);

        output.index += 1;
    }

    /// Updates the front of the population using the scores of the individuals. The front is a collection
    /// of individuals that are not dominated by any other individual in the population. This method is only
    /// called if the objective is multi-objective, as the front is not relevant for single-objective optimization.
    /// The front is updated in a separate thread to avoid blocking the main thread while the front is being calculated.
    /// This can significantly speed up the calculation of the front for large populations.
    fn update_front(&self, output: &mut EngineContext<C, T>) {
        let objective = self.objective();
        let thread_pool = self.thread_pool();

        if let Objective::Multi(_) = objective {
            let timer = Timer::new();
            let scores = output
                .population
                .iter()
                .map(|individual| individual.score().unwrap().clone())
                .collect::<Vec<Score>>();

            let front = Arc::clone(&output.front);
            thread_pool.submit(move || {
                front.lock().unwrap().update_front(&scores);
            });

            output.upsert_operation(metric_names::FRONT, 1.0, timer.duration());
        }
    }

    /// Adds various metrics to the output context, including the age of individuals, the score of individuals,
    /// and the number of unique scores in the population. These metrics can be used to monitor the progress of
    /// the genetic algorithm and to identify potential issues or areas for improvement.
    ///
    /// The age of an individual is the number of generations it has survived, while the score of an individual
    /// is a measure of its fitness. The number of unique scores in the population is a measure of diversity, with
    /// a higher number indicating a more diverse population.
    fn update_metrics(&self, output: &mut EngineContext<C, T>) {
        let mut age_metric = Metric::new_value(metric_names::AGE);
        let mut score_metric = Metric::new_value(metric_names::SCORE);
        let mut size_values = Vec::with_capacity(output.population.len());
        let mut unique = Vec::with_capacity(output.population.len());
        let mut equal_members = 0;

        for i in 0..output.population.len() {
            let phenotype = &output.population[i];

            if i > 0 && phenotype.genotype() == output.population[i - 1].genotype() {
                equal_members += 1;
            }

            let age = phenotype.age(output.index);
            let score = phenotype.score().unwrap();
            let phenotype_size = phenotype
                .genotype()
                .iter()
                .map(|chromosome| chromosome.len())
                .sum::<usize>();

            age_metric.add_value(age as f32);
            score_metric.add_value(score.as_f32());
            unique.push(score.as_f32());
            size_values.push(phenotype_size as f32);
        }

        unique.dedup();

        let mut unique_metric = Metric::new_value(metric_names::UNIQUE_SCORES);
        let mut size_metric = Metric::new_distribution(metric_names::GENOME_SIZE);
        let mut equal_metric = Metric::new_value(metric_names::NUM_EQUAL);

        unique_metric.add_value(unique.len() as f32);
        size_metric.add_sequence(&size_values);
        equal_metric.add_value(equal_members as f32);

        output.upsert_metric(equal_metric);
        output.upsert_metric(age_metric);
        output.upsert_metric(score_metric);
        output.upsert_metric(unique_metric);
        output.upsert_metric(size_metric);
    }

    fn survivor_selector(&self) -> &Box<dyn Select<C>> {
        self.params.survivor_selector()
    }

    fn offspring_selector(&self) -> &Box<dyn Select<C>> {
        self.params.offspring_selector()
    }

    fn alters(&self) -> &[Box<dyn Alter<C>>] {
        self.params.alters()
    }

    fn problem(&self) -> Arc<dyn Problem<C, T>> {
        self.params.problem()
    }

    fn objective(&self) -> &Objective {
        self.params.objective()
    }

    fn survivor_count(&self) -> usize {
        self.params.survivor_count()
    }

    fn offspring_count(&self) -> usize {
        self.params.offspring_count()
    }

    fn max_age(&self) -> usize {
        self.params.max_age()
    }

    fn thread_pool(&self) -> &ThreadPool {
        self.params.thread_pool()
    }

    fn replace_strategy(&self) -> &Box<dyn ReplacementStrategy<C>> {
        self.params.replacement_strategy()
    }

    fn start(&self) -> EngineContext<C, T> {
        match self.params.population() {
            Some(population) => {
                let front = self.params.front().clone();
                let errors = self.params.errors().clone();
                EngineContext::new(population.clone(), front, errors)
            }
            None => {
                let err = match self.params.errors() {
                    Some(e) => Some(e.clone()),
                    None => Some(EngineError::PopulationError(
                        "Population is not set".to_string(),
                    )),
                };

                EngineContext::new(
                    Population::new(Vec::new()),
                    self.params.front().clone(),
                    err,
                )
            }
        }
    }

    fn stop(&self, output: &mut EngineContext<C, T>) -> EngineContext<C, T> {
        output.timer.stop();
        output.clone()
    }
}
