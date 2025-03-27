use super::codexes::Codex;
use super::context::SharedEngineContext;
use super::thread_pool::{ThreadPool, WaitGroup};
use super::{
    Alter, Audit, Distance, EngineContext, GeneticEngineParams, MetricSet, Problem,
    ReplacementStrategy, Species, random_provider, speciate,
};
use crate::engines::builder::GeneticEngineBuilder;
use crate::engines::domain::timer::Timer;
use crate::engines::genome::population::Population;
use crate::objectives::Objective;
use crate::{Chromosome, Metric, Select, Valid, metric_names};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

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
    T: Clone + Send + 'static,
{
    params: GeneticEngineParams<C, T>,
}

impl<C, T> GeneticEngine<C, T>
where
    C: Chromosome,
    T: Clone + Send,
{
    /// Create a new instance of the `GeneticEngine` struct with the given parameters.
    /// - `params`: An instance of `GeneticEngineParams` that holds configuration parameters for the genetic engine.
    pub fn new(params: GeneticEngineParams<C, T>) -> Self {
        GeneticEngine { params }
    }

    /// Initializes a `GeneticEngineParams` using the provided codex, which defines how individuals
    /// are represented in the population. Because the `Codex` is always needed, this
    /// is a convenience method that allows users to create a `GeneticEngineParams` instance
    /// which will then be 'built' resulting in a `GeneticEngine` instance.
    ///
    /// **Note** with this method, the `Codex` is supplied to the `GeneticEngineParams` and thus
    /// the `GeneticEngineParams` also will need a `FitnessFn` to be supplied before building.
    pub fn from_codex(codex: impl Codex<C, T> + 'static) -> GeneticEngineBuilder<C, T> {
        GeneticEngineBuilder::new().codex(codex)
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
        GeneticEngineBuilder::new().problem(problem)
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
            self.speciate(&mut ctx);

            let survivors = self.select_survivors(&mut ctx);
            let offspring = self.create_offspring(&mut ctx);

            self.recombine(&mut ctx, survivors, offspring);

            self.filter(&mut ctx);
            self.evaluate(&mut ctx);
            self.update_front(&mut ctx);
            self.audit(&mut ctx);

            let mut owened = EngineContext::from(ctx);

            if limit(&owened) {
                break self.stop(&mut owened);
            } else {
                ctx = SharedEngineContext::from(owened);
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
    fn evaluate(&self, handle: &mut SharedEngineContext<C, T>) {
        let objective = self.objective();
        let thread_pool = self.thread_pool();
        let timer = Timer::new();

        let mut work_results = Vec::new();
        for idx in 0..handle.population_len() {
            let mut pop = handle.population_lock();

            let individual = &mut pop[idx];
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
            let mut pop = handle.population_lock();
            pop[idx].set_genotype(genotype);
            pop[idx].set_score(Some(score));
        }

        handle.upsert_operation(metric_names::EVALUATION, count, timer);

        objective.sort(&mut handle.population_lock());
    }

    /// Speciates the population into species based on the genetic distance between individuals.
    fn speciate(&self, ctx: &mut SharedEngineContext<C, T>) {
        let distance = self.distance();
        let objective = self.objective();

        if let Some(distance) = distance {
            let timer = Timer::new();
            let mut distances = Vec::new();

            let mut population = ctx.population_lock();
            let mut species = ctx.species_lock();

            speciate::generate_mascots(&mut population, &mut species);

            for i in 0..population.len() {
                let mut found = false;
                for j in 0..species.len() {
                    let species = &species[j];
                    let dist = distance.distance(population[i].genotype(), species.mascot());
                    distances.push(dist);

                    if dist < distance.threshold() {
                        population[i].set_species_id(Some(species.id()));
                        found = true;
                        break;
                    }
                }

                if !found {
                    let phenotype = &population[i];
                    let genotype = phenotype.genotype().clone();
                    let score = phenotype.score().unwrap();
                    let new_species = Species::new(genotype, score.clone(), ctx.index());

                    population[i].set_species_id(Some(new_species.id()));
                    species.push(new_species);
                }
            }

            speciate::fitness_share(&mut population, &mut species, objective);

            let species_count = species.len() as f32;
            ctx.upsert_operation(metric_names::SPECIATION, species_count as f32, timer);
            ctx.upsert_distribution(metric_names::DISTANCE, &distances);
        }
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
    fn select_survivors(&self, ctx: &mut SharedEngineContext<C, T>) -> Population<C> {
        let selector = self.survivor_selector();
        let count = self.survivor_count();
        let objective = self.objective();

        let timer = Timer::new();
        let result = selector.select(&ctx.population_lock(), objective, count);

        ctx.upsert_operation(selector.name(), count as f32, timer);

        result
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
    fn create_offspring(&self, ctx: &mut SharedEngineContext<C, T>) -> Population<C> {
        let selector = self.offspring_selector();
        let count = self.offspring_count();
        let objective = self.objective();
        let alters = self.alters();

        if ctx.species_lock().is_empty() || random_provider::random::<f32>() < 0.01 {
            let timer = Timer::new();
            let mut offspring = selector.select(&ctx.population_lock(), objective, count);

            ctx.upsert_operation(selector.name(), count as f32, timer);
            objective.sort(&mut offspring);

            alters.iter().for_each(|alterer| {
                let mut metric_set = ctx.metrics_lock();
                alterer
                    .alter(&mut offspring, ctx.index())
                    .into_iter()
                    .for_each(|metric| {
                        metric_set.upsert(metric);
                    });
            });

            offspring
        } else {
            let mut offspring = Vec::new();
            let mut population = ctx.population_lock();
            let species = ctx.species_lock();

            for i in 0..species.len() {
                let species = &species[i];
                let timer = Timer::new();

                let count = (species.score().as_f32() * count as f32).round() as usize;
                let members = population.take(|pheno| pheno.species_id() == Some(species.id()));

                let mut selected = selector.select(&members, objective, count);

                ctx.upsert_operation(selector.name(), count as f32, timer);
                objective.sort(&mut selected);

                alters.iter().for_each(|alterer| {
                    let mut metric_set = ctx.metrics_lock();
                    alterer
                        .alter(&mut selected, ctx.index())
                        .into_iter()
                        .for_each(|metric| {
                            metric_set.upsert(metric);
                        });
                });

                offspring.extend(selected);
            }

            offspring.into_iter().collect::<Population<C>>()
        }
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
    fn filter(&self, ctx: &mut SharedEngineContext<C, T>) {
        let max_age = self.max_age();

        let generation = ctx.index();
        let mut population = ctx.population_lock();
        let mut species = ctx.species_lock();

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

                replacement.replace(i, generation, &mut population, encoder);
            }
        }

        let before_species = species.len();
        species.retain(|species| species.age(generation) < max_age);
        let species_count = (before_species - species.len()) as f32;

        let duration = timer.duration();
        ctx.upsert_operation(metric_names::SPECIES_FILTER, species_count, duration);
        ctx.upsert_operation(metric_names::AGE_FILTER, age_count, duration);
        ctx.upsert_operation(metric_names::INVALID_FILTER, invalid_count, duration);
    }

    /// Recombines the survivors and offspring populations to create the next generation. The survivors
    /// are the individuals from the previous generation that will survive to the next generation, while the
    /// offspring are the individuals that were selected from the previous generation then altered.
    /// This method combines the survivors and offspring populations into a single population that
    /// will be used in the next iteration of the genetic algorithm.
    fn recombine(
        &self,
        handle: &mut SharedEngineContext<C, T>,
        survivors: Population<C>,
        offspring: Population<C>,
    ) {
        (*handle.population_lock()) = survivors
            .into_iter()
            .chain(offspring)
            .collect::<Population<C>>();
    }

    /// Audits the current state of the genetic algorithm, updating the best individual found so far
    /// and calculating various metrics such as the age of individuals, the score of individuals, and the
    /// number of unique scores in the population. This method is called at the end of each generation.
    fn audit(&self, ctx: &mut SharedEngineContext<C, T>) {
        let audits = self.audits();
        let problem = self.problem();
        let objective = self.objective();

        let mut population = ctx.population_lock();

        let audit_metrics = audits
            .iter()
            .map(|audit| audit.audit(ctx.index(), &population))
            .flatten()
            .collect::<Vec<Metric>>();

        for metric in audit_metrics {
            ctx.upsert_metric(metric);
        }

        if !population.is_sorted {
            objective.sort(&mut population);
        }

        if let Some(best_score) = population[0].score() {
            let current_score = ctx
                .score
                .lock()
                .unwrap()
                .as_ref()
                .map_or(true, |current| objective.is_better(best_score, current));
            if current_score {
                ctx.update_score(best_score.clone());
                ctx.update_best(problem.decode(population[0].genotype()));
            }
        }

        ctx.update_index();
    }

    /// Updates the front of the population using the scores of the individuals. The front is a collection
    /// of individuals that are not dominated by any other individual in the population. This method is only
    /// called if the objective is multi-objective, as the front is not relevant for single-objective optimization.
    /// The front is updated in a separate thread to avoid blocking the main thread while the front is being calculated.
    /// This can significantly speed up the calculation of the front for large populations.
    fn update_front(&self, ctx: &mut SharedEngineContext<C, T>) {
        let objective = self.objective();

        if let Objective::Multi(_) = objective {
            let timer = Timer::new();

            let population = ctx.population_lock();
            let new_individuals = population
                .iter()
                .filter(|p| p.generation == ctx.index())
                .collect::<Vec<_>>();

            let count = ctx.front().update_front(new_individuals.as_slice());
            ctx.upsert_operation(metric_names::FRONT, count as f32, timer);
        }
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

    fn distance(&self) -> Option<Arc<dyn Distance<C>>> {
        self.params.distance()
    }

    fn audits(&self) -> &[Arc<dyn Audit<C>>] {
        self.params.audits()
    }

    fn replace_strategy(&self) -> &Box<dyn ReplacementStrategy<C>> {
        self.params.replacement_strategy()
    }

    fn start(&self) -> SharedEngineContext<C, T> {
        let population = self.params.population().clone();

        SharedEngineContext {
            population: Arc::new(Mutex::new(population.clone())),
            best: Arc::new(Mutex::new(self.problem().decode(population[0].genotype()))),
            index: Arc::new(Mutex::new(0)),
            timer: Arc::new(Mutex::new(Timer::new())),
            metrics: Arc::new(Mutex::new(MetricSet::new())),
            score: Arc::new(Mutex::new(None)),
            front: Arc::new(Mutex::new(self.params.front().clone())),
            species: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn stop(&self, output: &mut EngineContext<C, T>) -> EngineContext<C, T> {
        output.timer.stop();
        output.clone()
    }
}
