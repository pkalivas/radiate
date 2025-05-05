use crate::codexes::Codex;
use crate::genome::phenotype::Phenotype;
use crate::genome::population::Population;
use crate::objectives::Score;
use crate::objectives::{Objective, Optimize};
use crate::steps::{
    AuditStep, Evaluator, FilterStep, FrontStep, RecombineStep, SequentialEvaluator, SpeciateStep,
    WorkerPoolEvaluator,
};
use crate::thread_pool::ThreadPool;
use crate::{
    Alter, AlterAction, Audit, Crossover, EncodeReplace, EngineProblem, EngineStep, Front,
    Generation, MetricAudit, MultiObjectiveGeneration, Mutate, Problem, ReplacementStrategy,
    RouletteSelector, Select, TournamentSelector, pareto,
};
use crate::{Chromosome, EngineConfig, EvaluateStep, GeneticEngine, Pipeline};
use core::panic;
use radiate_alters::{UniformCrossover, UniformMutator};
use radiate_core::engine::Context;
use radiate_core::{Diversity, Ecosystem, Epoch, Genotype, MetricSet};
use radiate_error::{RadiateError, radiate_err};
use std::cmp::Ordering;
use std::ops::Range;
use std::sync::{Arc, RwLock};

pub trait EngineBuilder<C, T, E>
where
    C: Chromosome + 'static,
    T: Clone + Send + 'static,
    E: Epoch,
{
    fn add_parameter(&mut self, adder: impl FnOnce(&mut EngineParams<C, T>));
    fn build(self) -> GeneticEngine<C, T, E>;
}

#[derive(Clone)]
pub struct EngineParams<C, T = Genotype<C>>
where
    C: Chromosome + 'static,
    T: Clone + 'static,
{
    pub population_size: usize,
    pub max_age: usize,
    pub front_range: Range<usize>,
    pub offspring_fraction: f32,
    pub species_threshold: f32,
    pub max_species_age: usize,
    pub thread_pool: Arc<ThreadPool>,
    pub objective: Objective,
    pub survivor_selector: Arc<dyn Select<C>>,
    pub offspring_selector: Arc<dyn Select<C>>,
    pub diversity: Option<Arc<dyn Diversity<C>>>,
    pub alterers: Vec<Arc<dyn Alter<C>>>,
    pub audits: Vec<Arc<dyn Audit<C>>>,
    pub population: Option<Population<C>>,
    pub codex: Option<Arc<dyn Codex<C, T>>>,
    pub evaluator: Arc<dyn Evaluator<C, T>>,
    pub fitness_fn: Option<Arc<dyn Fn(T) -> Score + Send + Sync>>,
    pub problem: Option<Arc<dyn Problem<C, T>>>,
    pub encoder: Option<Arc<dyn Fn() -> Genotype<C>>>,
    pub replacement_strategy: Arc<dyn ReplacementStrategy<C>>,
    pub front: Option<Front<Phenotype<C>>>,
}

/// Parameters for the genetic engine.
/// This struct is used to configure the genetic engine before it is created.
///
/// When the `GeneticEngineParams`  calls the `build` method, it will create a new instance
/// of the `GeneticEngine` with the given parameters. If any of the required parameters are not
/// set, the `build` method will panic. At a minimum, the `codex` and `fitness_fn` must be set.
/// The `GeneticEngineParams` struct is a builder pattern that allows you to set the parameters of
/// the `GeneticEngine` in a fluent and functional way.
///
/// # Type Parameters
/// - `C`: The type of chromosome used in the genotype, which must implement the `Chromosome` trait.
/// - `T`: The type of the best individual in the population.
///
#[derive(Clone)]
pub struct GeneticEngineBuilder<C, T, E = Generation<C, T>>
where
    C: Chromosome + 'static,
    T: Clone + 'static,
    E: Epoch,
{
    pub(crate) params: EngineParams<C, T>,
    errors: Vec<RadiateError>,
    _epoch: std::marker::PhantomData<E>,
}

impl<C, T, E> GeneticEngineBuilder<C, T, E>
where
    C: Chromosome,
    T: Clone + Send,
    E: Epoch,
{
    /// Set the population size of the genetic engine. Default is 100.
    pub fn population_size(mut self, population_size: usize) -> Self {
        if population_size < 1 {
            self.errors
                .push(radiate_err!(InvalidConfig: "population_size must be greater than 0"));
        }

        self.params.population_size = population_size;
        self
    }

    /// Set the maximum age of an individual in the population. Default is 25.
    pub fn max_age(mut self, max_age: usize) -> Self {
        if max_age < 1 {
            self.errors
                .push(radiate_err!(InvalidConfig: "max_age must be greater than 0"));
        }

        self.params.max_age = max_age;
        self
    }

    pub fn diversity<D: Diversity<C> + 'static>(mut self, diversity: D) -> Self {
        self.params.diversity = Some(Arc::new(diversity));
        self
    }

    pub fn species_threshold(mut self, threshold: f32) -> Self {
        if threshold < 0.0 {
            self.errors
                .push(radiate_err!(InvalidConfig: "species_threshold must be greater than 0"));
        }

        self.params.species_threshold = threshold;
        self
    }

    pub fn max_species_age(mut self, max_species_age: usize) -> Self {
        if max_species_age < 1 {
            self.errors.push(radiate_err!(
                InvalidConfig: "max_species_age must be greater than 0"
            ));
        }

        self.params.max_species_age = max_species_age;
        self
    }

    /// The `FilterStrategy` is used to determine how a new individual is added to the `Population`
    /// if an individual is deemed to be either invalid or reaches the maximum age.
    ///
    /// Default is `FilterStrategy::Encode`, which means that a new individual will be created
    /// be using the `Codex` to encode a new individual from scratch.
    pub fn replace_strategy<R: ReplacementStrategy<C> + 'static>(mut self, replace: R) -> Self {
        self.params.replacement_strategy = Arc::new(replace);
        self
    }

    /// Add a single audit to the algorithm that will produce additional metrics
    /// to collect during the evolution process.
    pub fn audit(mut self, audit: impl Audit<C> + 'static) -> Self {
        self.params.audits.push(Arc::new(audit));
        self
    }

    /// Add a list of audits to the algorithm that will produce additional metrics
    /// to collect during the evolution process.
    pub fn audits(mut self, audits: Vec<Arc<dyn Audit<C>>>) -> Self {
        self.params.audits.extend(audits);
        self
    }

    pub fn evaluator<EV: Evaluator<C, T> + 'static>(mut self, evaluator: EV) -> Self {
        self.params.evaluator = Arc::new(evaluator);
        self
    }

    /// Set the fraction of the population that will be replaced by offspring each generation.
    /// Default is 0.8. This is a value from 0...=1 that represents the fraction of
    /// population that will be replaced by offspring each generation. The remainder will 'survive' to the next generation.
    pub fn offspring_fraction(mut self, offspring_fraction: f32) -> Self {
        if !(0.0..=1.0).contains(&offspring_fraction) {
            self.errors.push(radiate_err!(
                InvalidConfig: "offspring_fraction must be between 0.0 and 1.0"
            ));
        }

        self.params.offspring_fraction = offspring_fraction;
        self
    }

    /// Set the codex that will be used to encode and decode the genotype of the population.
    pub fn codex<D: Codex<C, T> + 'static>(mut self, codex: D) -> Self {
        self.params.codex = Some(Arc::new(codex));
        self
    }

    /// Set the problem of the genetic engine. This is useful if you want to provide a custom problem.
    pub fn problem<P: Problem<C, T> + 'static>(mut self, problem: P) -> Self {
        self.params.problem = Some(Arc::new(problem));
        self
    }

    /// Set the population of the genetic engine. This is useful if you want to provide a custom population.
    /// If this is not set, the genetic engine will create a new population of `population_size` using the codex.
    pub fn population(mut self, population: Population<C>) -> Self {
        self.params.population = Some(population);
        self
    }

    /// Set the fitness function of the genetic engine. This is the function that will be
    /// used to evaluate the fitness of each individual in the population. This function should
    /// take a single argument of type T and return a `Score`. The `Score` is used to
    /// evaluate or rank the fitness of the individual.
    ///
    /// This method is required and must be set before calling the `build` method.
    pub fn fitness_fn<S: Into<Score>>(
        mut self,
        fitness_func: impl Fn(T) -> S + Send + Sync + 'static,
    ) -> Self {
        let other = move |x| fitness_func(x).into();
        self.params.fitness_fn = Some(Arc::new(other));
        self
    }

    /// Set the survivor selector of the genetic engine. This is the selector that will
    /// be used to select the survivors of the population. Default is `TournamentSelector`
    /// with a group size of 3.
    pub fn survivor_selector<S: Select<C> + 'static>(mut self, selector: S) -> Self {
        self.params.survivor_selector = Arc::new(selector);
        self
    }

    /// Set the offspring selector of the genetic engine. This is the selector that will
    /// be used to select the offspring of the population. Default is `RouletteSelector`.
    pub fn offspring_selector<S: Select<C> + 'static>(mut self, selector: S) -> Self {
        self.params.offspring_selector = Arc::new(selector);
        self
    }

    /// Set the alterer of the genetic engine. This is the alterer that will be used to
    /// alter the offspring of the population. The alterer is used to apply mutations
    /// and crossover operations to the offspring and will be used to create the next
    /// generation of the population. **Note**: the order of the alterers is important - the
    /// alterers will be applied in the order they are provided.
    pub fn alter(mut self, alterers: Vec<Box<dyn Alter<C>>>) -> Self {
        self.params.alterers = alterers.into_iter().map(|alt| alt.into()).collect();
        self
    }

    /// Define a single mutator for the genetic engine - this will be  converted to
    /// a `Box<dyn Alter<C>>` and added to the list of alterers. Note: The order in which
    /// mutators and crossovers are added is the order in which they will be applied during
    /// the evolution process.
    pub fn mutator<M: Mutate<C> + 'static>(mut self, mutator: M) -> Self {
        self.params.alterers.push(Arc::new(mutator.alterer()));
        self
    }

    /// Define a list of mutators for the genetic engine - this will be converted to a list
    /// of `Box<dyn Alter<C>>` and added to the list of alterers. Just like adding a single mutator,
    /// the order in which mutators and crossovers are added is the order in which they will be applied
    /// during the evolution process.s
    pub fn mutators(mut self, mutators: Vec<Box<dyn Mutate<C>>>) -> Self {
        let mutate_actions = mutators
            .into_iter()
            .map(|m| Arc::new(AlterAction::Mutate(m.name(), m.rate(), m)) as Arc<dyn Alter<C>>)
            .collect::<Vec<_>>();

        self.params.alterers.extend(mutate_actions);
        self
    }

    /// Define a single crossover for the genetic engine - this will be converted to
    /// a `Box<dyn Alter<C>>` and added to the list of alterers. Note: The order in which
    /// mutators and crossovers are added is the order in which they will be applied during
    /// the evolution process.s
    pub fn crossover<R: Crossover<C> + 'static>(mut self, crossover: R) -> Self {
        self.params.alterers.push(Arc::new(crossover.alterer()));
        self
    }

    /// Define a list of crossovers for the genetic engine - this will be converted to a list
    /// of `Box<dyn Alter<C>>` and added to the list of alterers. Just like adding a single crossover,
    /// the order in which mutators and crossovers are added is the order in which they will be applied
    /// during the evolution process.
    pub fn crossovers(mut self, crossovers: Vec<Box<dyn Crossover<C>>>) -> Self {
        let crossover_actions = crossovers
            .into_iter()
            .map(|c| Arc::new(AlterAction::Crossover(c.name(), c.rate(), c)) as Arc<dyn Alter<C>>)
            .collect::<Vec<_>>();

        self.params.alterers.extend(crossover_actions);
        self
    }

    /// Set the optimization goal of the genetic engine to minimize the fitness function.
    pub fn minimizing(mut self) -> Self {
        self.params.objective = Objective::Single(Optimize::Minimize);
        self
    }

    /// Set the optimization goal of the genetic engine to maximize the fitness function.
    pub fn maximizing(mut self) -> Self {
        self.params.objective = Objective::Single(Optimize::Maximize);
        self
    }

    pub fn multi_objective(
        mut self,
        objectives: Vec<Optimize>,
    ) -> GeneticEngineBuilder<C, T, MultiObjectiveGeneration<C>> {
        self.params.objective = Objective::Multi(objectives);
        GeneticEngineBuilder {
            params: self.params,
            errors: self.errors,
            _epoch: std::marker::PhantomData,
        }
    }

    /// Set the minimum and maximum size of the pareto front. This is used for
    /// multi-objective optimization problems where the goal is to find the best
    /// solutions that are not dominated by any other solution.
    pub fn front_size(
        mut self,
        range: Range<usize>,
    ) -> GeneticEngineBuilder<C, T, MultiObjectiveGeneration<C>> {
        self.params.front_range = range;
        GeneticEngineBuilder {
            params: self.params,
            errors: self.errors,
            _epoch: std::marker::PhantomData,
        }
    }

    /// Set the thread pool of the genetic engine. This is the thread pool that will be used
    /// to execute the fitness function in parallel. Some fitness functions may be computationally
    /// expensive and can benefit from parallel execution.
    pub fn num_threads(mut self, num_threads: usize) -> Self
    where
        T: Send + Sync,
    {
        self.params.thread_pool = Arc::new(ThreadPool::new(num_threads));
        self.params.evaluator = Arc::new(WorkerPoolEvaluator);
        self
    }

    /// Build the genetic engine with the given parameters. This will create a new
    /// instance of the `GeneticEngine` with the given parameters.
    pub fn build(mut self) -> GeneticEngine<C, T, E> {
        if self.params.problem.is_none() {
            if self.params.codex.is_none() {
                panic!("Codex not set");
            }

            if self.params.fitness_fn.is_none() {
                panic!("Fitness function not set");
            }

            let problem = EngineProblem {
                codex: self.params.codex.clone().unwrap(),
                fitness_fn: self.params.fitness_fn.clone().unwrap(),
            };

            self.problem(problem).build()
        } else {
            self.build_population();
            self.build_alterer();
            self.build_front();

            let config = EngineConfig {
                population: self.params.population.clone().unwrap(),
                problem: self.params.problem.clone().unwrap(),
                survivor_selector: self.params.survivor_selector.clone(),
                offspring_selector: self.params.offspring_selector.clone(),
                replacement_strategy: self.params.replacement_strategy.clone(),
                audits: self.params.audits.clone(),
                alterers: self.params.alterers.clone(),
                objective: self.params.objective.clone(),
                thread_pool: self.params.thread_pool.clone(),
                max_age: self.params.max_age,
                max_species_age: self.params.max_species_age,
                species_threshold: self.params.species_threshold,
                diversity: self.params.diversity.clone(),
                front: Arc::new(RwLock::new(self.params.front.clone().unwrap())),
                offspring_fraction: self.params.offspring_fraction,
                evaluator: self.params.evaluator.clone(),
            };

            let mut pipeline = Pipeline::<C>::default();

            pipeline.add_step(Self::build_eval_step(&config));
            pipeline.add_step(Self::build_recombine_step(&config));
            pipeline.add_step(Self::build_filter_step(&config));
            pipeline.add_step(Self::build_eval_step(&config));
            pipeline.add_step(Self::build_front_step(&config));
            pipeline.add_step(Self::build_species_step(&config));
            pipeline.add_step(Self::build_audit_step(&config));

            let context = Context {
                ecosystem: Ecosystem::new(config.population.clone()),
                best: config.problem.decode(config.population()[0].genotype()),
                index: 0,
                metrics: MetricSet::new(),
                score: None,
                front: config.front.clone(),
                objective: config.objective.clone(),
                problem: config.problem.clone(),
            };

            GeneticEngine::<C, T, E>::new(context, pipeline)
        }
    }

    fn build_eval_step(config: &EngineConfig<C, T>) -> Option<Box<dyn EngineStep<C>>> {
        let evaluator = config.evaluator.clone();
        let eval_step = EvaluateStep {
            objective: config.objective.clone(),
            thread_pool: config.thread_pool.clone(),
            problem: config.problem.clone(),
            evaluator: evaluator.clone(),
        };

        Some(Box::new(eval_step))
    }

    fn build_recombine_step(config: &EngineConfig<C, T>) -> Option<Box<dyn EngineStep<C>>> {
        let recombine_step = RecombineStep {
            survivor_selector: config.survivor_selector(),
            offspring_selector: config.offspring_selector(),
            alters: config.alters().to_vec(),
            survivor_count: config.survivor_count(),
            offspring_count: config.offspring_count(),
            objective: config.objective(),
        };

        Some(Box::new(recombine_step))
    }

    fn build_filter_step(config: &EngineConfig<C, T>) -> Option<Box<dyn EngineStep<C>>> {
        let filter_step = FilterStep {
            replacer: config.replacement_strategy(),
            encoder: config.encoder(),
            max_age: config.max_age(),
            max_species_age: config.max_species_age(),
        };

        Some(Box::new(filter_step))
    }

    fn build_audit_step(config: &EngineConfig<C, T>) -> Option<Box<dyn EngineStep<C>>> {
        if config.audits().is_empty() {
            return None;
        }

        let audit_step = AuditStep {
            audits: config.audits().to_vec(),
        };

        Some(Box::new(audit_step))
    }

    fn build_front_step(config: &EngineConfig<C, T>) -> Option<Box<dyn EngineStep<C>>> {
        if let Objective::Single(_) = config.objective() {
            return None;
        }

        let front_step = FrontStep {
            front: config.front().clone(),
        };

        Some(Box::new(front_step))
    }

    fn build_species_step(config: &EngineConfig<C, T>) -> Option<Box<dyn EngineStep<C>>> {
        if config.diversity().is_none() {
            return None;
        }

        let species_step = SpeciateStep {
            threashold: config.species_threshold(),
            diversity: config.diversity().clone().unwrap(),
            thread_pool: config.thread_pool(),
            objective: config.objective(),
        };

        Some(Box::new(species_step))
    }

    /// Build the population of the genetic engine. This will create a new population
    /// using the codex if the population is not set.
    fn build_population(&mut self) {
        self.params.population = match &self.params.population {
            None => Some(match self.params.problem.as_ref() {
                Some(problem) => Population::from((self.params.population_size, || {
                    Phenotype::from((problem.encode(), 0))
                })),
                None => panic!("Codex not set"),
            }),
            Some(pop) => Some(pop.clone()),
        };
    }

    /// Build the alterer of the genetic engine. This will create a
    /// new `UniformCrossover` and `UniformMutator` if the alterer is not set.
    /// with a 0.5 crossover rate and a 0.1 mutation rate.
    fn build_alterer(&mut self) {
        if !self.params.alterers.is_empty() {
            return;
        }

        let crossover = Arc::new(UniformCrossover::new(0.5).alterer()) as Arc<dyn Alter<C>>;
        let mutator = Arc::new(UniformMutator::new(0.1).alterer()) as Arc<dyn Alter<C>>;

        self.params.alterers.push(crossover);
        self.params.alterers.push(mutator);
    }

    /// Build the pareto front of the genetic engine. This will create a new `Front`
    /// if the front is not set. The `Front` is used to store the best individuals
    /// in the population and is used for multi-objective optimization problems.
    fn build_front(&mut self) {
        if self.params.front.is_some() {
            return;
        }

        let front_obj = self.params.objective.clone();
        self.params.front = Some(Front::new(
            self.params.front_range.clone(),
            front_obj.clone(),
            self.params.thread_pool.clone(),
            move |one: &Phenotype<C>, two: &Phenotype<C>| {
                if one.score().is_none() || two.score().is_none() {
                    return Ordering::Equal;
                }

                if let (Some(one), Some(two)) = (one.score(), two.score()) {
                    if pareto::dominance(one, two, &front_obj) {
                        return Ordering::Greater;
                    } else if pareto::dominance(two, one, &front_obj) {
                        return Ordering::Less;
                    }
                }

                Ordering::Equal
            },
        ));
    }
}

impl<C, T, E> EngineBuilder<C, T, E> for GeneticEngineBuilder<C, T, E>
where
    C: Chromosome + 'static,
    T: Clone + Send + 'static,
    E: Epoch,
{
    fn add_parameter(&mut self, adder: impl FnOnce(&mut EngineParams<C, T>)) {
        adder(&mut self.params);
    }

    fn build(self) -> GeneticEngine<C, T, E> {
        if !self.errors.is_empty() {
            panic!("Errors found in builder: {:?}", self.errors);
        }

        self.build()
    }
}

impl<C, T, E> Default for GeneticEngineBuilder<C, T, E>
where
    C: Chromosome + 'static,
    T: Clone + Send + 'static,
    E: Epoch,
{
    fn default() -> Self {
        GeneticEngineBuilder {
            params: EngineParams {
                population_size: 100,
                max_age: 20,
                offspring_fraction: 0.8,
                front_range: 800..900,
                thread_pool: Arc::new(ThreadPool::new(1)),
                objective: Objective::Single(Optimize::Maximize),
                survivor_selector: Arc::new(TournamentSelector::new(3)),
                offspring_selector: Arc::new(RouletteSelector::new()),
                replacement_strategy: Arc::new(EncodeReplace),
                audits: vec![Arc::new(MetricAudit)],
                alterers: Vec::new(),
                species_threshold: 1.5,
                max_species_age: 25,
                evaluator: Arc::new(SequentialEvaluator),
                encoder: None,
                diversity: None,
                codex: None,
                population: None,
                fitness_fn: None,
                problem: None,
                front: None,
            },
            errors: Vec::new(),
            _epoch: std::marker::PhantomData,
        }
    }
}

impl<C, T, E> From<EngineParams<C, T>> for GeneticEngineBuilder<C, T, E>
where
    C: Chromosome + 'static,
    T: Clone + Send + 'static,
    E: Epoch,
{
    fn from(params: EngineParams<C, T>) -> Self {
        GeneticEngineBuilder {
            params,
            errors: Vec::new(),
            _epoch: std::marker::PhantomData,
        }
    }
}
