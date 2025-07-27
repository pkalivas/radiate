mod alters;
mod evaluators;
mod objectives;
mod population;
mod problem;
mod selectors;
mod species;

use crate::builder::evaluators::EvaluationParams;
use crate::builder::objectives::OptimizeParams;
use crate::builder::population::PopulationParams;
use crate::builder::selectors::SelectionParams;
use crate::builder::species::SpeciesParams;
use crate::codecs::Codec;
use crate::genome::phenotype::Phenotype;
use crate::genome::population::Population;
use crate::objectives::Score;
use crate::objectives::{Objective, Optimize};
use crate::pipeline::Pipeline;
use crate::steps::{AuditStep, EngineStep, FilterStep, FrontStep, RecombineStep, SpeciateStep};
use crate::{
    Alter, Context, Crossover, EncodeReplace, EngineEvent, EngineProblem, EventBus, EventHandler,
    Front, Mutate, Problem, ReplacementStrategy, RouletteSelector, Select, TournamentSelector,
    pareto,
};
use crate::{Chromosome, EvaluateStep, GeneticEngine};
use core::panic;
use radiate_alters::{UniformCrossover, UniformMutator};
use radiate_core::{
    Diversity, Ecosystem, Evaluator, Executor, FitnessEvaluator, Genotype, MetricSet,
};
use radiate_error::RadiateError;
use std::cmp::Ordering;
use std::sync::{Arc, Mutex, RwLock};

#[derive(Clone)]
pub struct EngineParams<C, T>
where
    C: Chromosome + 'static,
    T: Clone + 'static,
{
    pub population_params: PopulationParams<C>,
    pub evaluation_params: EvaluationParams<C, T>,
    pub species_params: SpeciesParams<C>,
    pub selection_params: SelectionParams<C>,
    pub optimization_params: OptimizeParams<C>,

    pub alterers: Vec<Arc<dyn Alter<C>>>,
    pub codec: Option<Arc<dyn Codec<C, T>>>,
    pub fitness_fn: Option<Arc<dyn Fn(T) -> Score + Send + Sync>>,
    pub problem: Option<Arc<dyn Problem<C, T>>>,
    pub replacement_strategy: Arc<dyn ReplacementStrategy<C>>,
    pub handlers: Vec<Arc<Mutex<dyn EventHandler<EngineEvent<T>>>>>,
}

/// Parameters for the genetic engine.
/// This struct is used to configure the genetic engine before it is created.
///
/// When the `GeneticEngineParams`  calls the `build` method, it will create a new instance
/// of the `GeneticEngine` with the given parameters. If any of the required parameters are not
/// set, the `build` method will panic. At a minimum, the `codec` and `fitness_fn` must be set.
/// The `GeneticEngineParams` struct is a builder pattern that allows you to set the parameters of
/// the `GeneticEngine` in a fluent and functional way.
///
/// # Type Parameters
/// - `C`: The type of chromosome used in the genotype, which must implement the `Chromosome` trait.
/// - `T`: The type of the best individual in the population.
#[derive(Clone)]
pub struct GeneticEngineBuilder<C, T>
where
    C: Chromosome + Clone + 'static,
    T: Clone + 'static,
{
    params: EngineParams<C, T>,
    errors: Vec<RadiateError>,
}

impl<C, T> GeneticEngineBuilder<C, T>
where
    C: Chromosome + PartialEq + Clone,
    T: Clone + Send,
{
    /// The `FilterStrategy` is used to determine how a new individual is added to the `Population`
    /// if an individual is deemed to be either invalid or reaches the maximum age.
    ///
    /// Default is `FilterStrategy::Encode`, which means that a new individual will be created
    /// be using the `Codec` to encode a new individual from scratch.
    pub fn replace_strategy<R: ReplacementStrategy<C> + 'static>(mut self, replace: R) -> Self {
        self.params.replacement_strategy = Arc::new(replace);
        self
    }

    pub fn subscribe<H>(mut self, handler: H) -> Self
    where
        H: EventHandler<EngineEvent<T>> + 'static,
    {
        self.params.handlers.push(Arc::new(Mutex::new(handler)));
        self
    }

    pub fn with_values<F>(mut self, f: F) -> Self
    where
        F: FnOnce(&mut EngineParams<C, T>),
    {
        f(&mut self.params);
        self
    }
}

/// Static step builder for the genetic engine.
impl<C, T> GeneticEngineBuilder<C, T>
where
    C: Chromosome + Clone + PartialEq + 'static,
    T: Clone + Send + Sync + 'static,
{
    /// Build the genetic engine with the given parameters. This will create a new
    /// instance of the `GeneticEngine` with the given parameters.
    pub fn build(mut self) -> GeneticEngine<C, T> {
        if self.params.problem.is_none() {
            if self.params.codec.is_none() {
                panic!("Codec not set");
            }

            if self.params.fitness_fn.is_none() {
                panic!("Fitness function not set");
            }

            let problem = EngineProblem {
                codec: self.params.codec.clone().unwrap(),
                fitness_fn: self.params.fitness_fn.clone().unwrap(),
            };

            self.problem(problem).build()
        } else {
            self.build_population();
            self.build_alterer();
            self.build_front();

            let config = EngineConfig::<C, T>::from(&self.params);

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
                epoch_metrics: MetricSet::new(),
                score: None,
                front: config.front.clone(),
                objective: config.objective.clone(),
                problem: config.problem.clone(),
            };

            let event_bus = EventBus::new(
                config.executor().bus_executor.clone(),
                self.params.handlers.clone(),
            );

            GeneticEngine::<C, T>::new(context, pipeline, event_bus)
        }
    }

    fn build_eval_step(config: &EngineConfig<C, T>) -> Option<Box<dyn EngineStep<C>>> {
        let evaluator = config.evaluator.clone();

        let eval_step = EvaluateStep {
            objective: config.objective.clone(),
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

    fn build_audit_step(_: &EngineConfig<C, T>) -> Option<Box<dyn EngineStep<C>>> {
        Some(Box::new(AuditStep))
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
            executor: config.executor().species_executor.clone(),
            objective: config.objective(),
        };

        Some(Box::new(species_step))
    }

    /// Build the population of the genetic engine. This will create a new population
    /// using the codec if the population is not set.
    fn build_population(&mut self) {
        self.params.population_params.population = match &self.params.population_params.population {
            None => Some(match self.params.problem.as_ref() {
                Some(problem) => {
                    Population::from((self.params.population_params.population_size, || {
                        Phenotype::from((problem.encode(), 0))
                    }))
                }
                None => panic!("Codec not set"),
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
        if self.params.optimization_params.front.is_some() {
            return;
        }

        let front_obj = self.params.optimization_params.objectives.clone();
        self.params.optimization_params.front = Some(Front::new(
            self.params.optimization_params.front_range.clone(),
            front_obj.clone(),
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

impl<C, T> Default for GeneticEngineBuilder<C, T>
where
    C: Chromosome + Clone + 'static,
    T: Clone + Send + 'static,
{
    fn default() -> Self {
        GeneticEngineBuilder {
            params: EngineParams {
                population_params: PopulationParams {
                    population_size: 100,
                    max_age: 20,
                    population: None,
                },
                species_params: SpeciesParams {
                    diversity: None,
                    species_threshold: 0.5,
                    max_species_age: 25,
                },
                evaluation_params: EvaluationParams {
                    evaluator: Arc::new(FitnessEvaluator::new(Arc::new(Executor::default()))),
                    fitness_executor: Arc::new(Executor::default()),
                    species_executor: Arc::new(Executor::default()),
                    bus_executor: Arc::new(Executor::default()),
                },
                selection_params: SelectionParams {
                    offspring_fraction: 0.8,
                    survivor_selector: Arc::new(TournamentSelector::new(3)),
                    offspring_selector: Arc::new(RouletteSelector::new()),
                },
                optimization_params: OptimizeParams {
                    objectives: Objective::Single(Optimize::Maximize),
                    front_range: 800..900,
                    front: None,
                },

                replacement_strategy: Arc::new(EncodeReplace),
                alterers: Vec::new(),
                codec: None,
                fitness_fn: None,
                problem: None,
                handlers: Vec::new(),
            },
            errors: Vec::new(),
        }
    }
}

#[derive(Clone)]
struct EngineConfig<C: Chromosome, T: Clone> {
    population: Population<C>,
    problem: Arc<dyn Problem<C, T>>,
    survivor_selector: Arc<dyn Select<C>>,
    offspring_selector: Arc<dyn Select<C>>,
    replacement_strategy: Arc<dyn ReplacementStrategy<C>>,
    alterers: Vec<Arc<dyn Alter<C>>>,
    species_threshold: f32,
    diversity: Option<Arc<dyn Diversity<C>>>,
    evaluator: Arc<dyn Evaluator<C, T>>,
    objective: Objective,
    max_age: usize,
    max_species_age: usize,
    front: Arc<RwLock<Front<Phenotype<C>>>>,
    offspring_fraction: f32,
    executor: EvaluationParams<C, T>,
}

impl<C: Chromosome, T: Clone> EngineConfig<C, T> {
    pub fn population(&self) -> &Population<C> {
        &self.population
    }

    pub fn survivor_selector(&self) -> Arc<dyn Select<C>> {
        Arc::clone(&self.survivor_selector)
    }

    pub fn offspring_selector(&self) -> Arc<dyn Select<C>> {
        Arc::clone(&self.offspring_selector)
    }

    pub fn replacement_strategy(&self) -> Arc<dyn ReplacementStrategy<C>> {
        Arc::clone(&self.replacement_strategy)
    }

    pub fn alters(&self) -> &[Arc<dyn Alter<C>>] {
        &self.alterers
    }

    pub fn objective(&self) -> Objective {
        self.objective.clone()
    }

    pub fn max_age(&self) -> usize {
        self.max_age
    }

    pub fn max_species_age(&self) -> usize {
        self.max_species_age
    }

    pub fn species_threshold(&self) -> f32 {
        self.species_threshold
    }

    pub fn diversity(&self) -> Option<Arc<dyn Diversity<C>>> {
        self.diversity.clone()
    }

    pub fn front(&self) -> Arc<RwLock<Front<Phenotype<C>>>> {
        Arc::clone(&self.front)
    }

    pub fn survivor_count(&self) -> usize {
        self.population.len() - self.offspring_count()
    }

    pub fn offspring_count(&self) -> usize {
        (self.population.len() as f32 * self.offspring_fraction) as usize
    }

    pub fn executor(&self) -> &EvaluationParams<C, T> {
        &self.executor
    }

    pub fn encoder(&self) -> Arc<dyn Fn() -> Genotype<C> + Send + Sync>
    where
        C: 'static,
        T: 'static,
    {
        let problem = Arc::clone(&self.problem);
        Arc::new(move || problem.encode())
    }
}

impl<C, T> From<&EngineParams<C, T>> for EngineConfig<C, T>
where
    C: Chromosome + Clone + 'static,
    T: Clone + Send + Sync + 'static,
{
    fn from(params: &EngineParams<C, T>) -> Self {
        Self {
            population: params.population_params.population.clone().unwrap(),
            problem: params.problem.clone().unwrap(),
            survivor_selector: params.selection_params.survivor_selector.clone(),
            offspring_selector: params.selection_params.offspring_selector.clone(),
            replacement_strategy: params.replacement_strategy.clone(),
            alterers: params.alterers.clone(),
            objective: params.optimization_params.objectives.clone(),
            max_age: params.population_params.max_age,
            max_species_age: params.species_params.max_species_age,
            species_threshold: params.species_params.species_threshold,
            diversity: params.species_params.diversity.clone(),
            front: Arc::new(RwLock::new(
                params.optimization_params.front.clone().unwrap(),
            )),
            offspring_fraction: params.selection_params.offspring_fraction,
            evaluator: params.evaluation_params.evaluator.clone(),
            executor: params.evaluation_params.clone(),
        }
    }
}

impl<C, T> std::fmt::Debug for EngineConfig<C, T>
where
    C: Chromosome + Clone + 'static,
    T: Clone + Send + Sync + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EngineConfig")
            .field("population_size", &self.population.len())
            .field("problem", &"EngineProblem")
            .field(
                "survivor_selector",
                &self.survivor_selector.name().to_string(),
            )
            .field(
                "offspring_selector",
                &self.offspring_selector.name().to_string(),
            )
            .field("alterers", &self.alterers.len())
            .field("objective", &self.objective)
            .field("max_age", &self.max_age)
            .field("max_species_age", &self.max_species_age)
            .field("species_threshold", &self.species_threshold)
            .field(
                "diversity",
                if self.diversity.is_some() {
                    &"Some(Diversity)"
                } else {
                    &"None"
                },
            )
            .field("front_range", &self.front.read().unwrap().range())
            .field("offspring_fraction", &self.offspring_fraction)
            .finish()
    }
}
