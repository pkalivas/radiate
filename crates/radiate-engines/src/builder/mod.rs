mod alters;
pub(crate) mod config;
mod evaluators;
mod objectives;
mod population;
mod problem;
mod selectors;
mod species;

use crate::builder::evaluators::EvaluationParams;
use crate::builder::objectives::OptimizeParams;
use crate::builder::population::PopulationParams;
use crate::builder::problem::ProblemParams;
use crate::builder::selectors::SelectionParams;
use crate::builder::species::SpeciesParams;
use crate::genome::phenotype::Phenotype;
#[cfg(feature = "serde")]
use crate::io::FileReader;
use crate::objectives::{Objective, Optimize};
use crate::pipeline::Pipeline;
use crate::steps::{
    EngineStep, FilterStep, FrontStep, MetricStep, RecombineStep, SelectConfig, SpeciateStep,
};
use crate::{Chromosome, EvaluateStep, GeneticEngine};
use crate::{
    Crossover, EncodeReplace, EventBus, EventHandler, Front, Mutate, ReplacementStrategy,
    RouletteSelector, TournamentSelector, context::EvolutionContext,
};
use crate::{Generation, Result};
use config::EngineConfig;
use radiate_alters::{UniformCrossover, UniformMutator};
use radiate_core::MetricQuery;
use radiate_core::evaluator::BatchFitnessEvaluator;
use radiate_core::problem::{BatchEngineProblem, EngineProblem};
use radiate_core::{Alterer, Ecosystem, Executor, FitnessEvaluator, Rate, Valid};
use radiate_core::{RadiateError, ensure, radiate_err};
use radiate_utils::VersionedCounts;
#[cfg(feature = "serde")]
use serde::Deserialize;
use std::sync::{Arc, Mutex};

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
    pub problem_params: ProblemParams<C, T>,

    pub alterers: Vec<Alterer<C>>,
    pub replacement_strategy: Arc<dyn ReplacementStrategy<C>>,
    pub handlers: Vec<Arc<Mutex<dyn EventHandler<T>>>>,
    pub generation: Option<Generation<C, T>>,
    pub exprs: Option<Arc<Mutex<Vec<MetricQuery>>>>,
}

/// Parameters for the genetic engine.
/// This struct is used to configure the genetic engine before it is created.
///
/// When the `GeneticEngineBuilder`  calls the `build` method, it will create a new instance
/// of the [GeneticEngine] with the given parameters. If any of the required parameters are not
/// set, the `build` method will panic. At a minimum, the `codec` and `fitness_fn` must be set.
/// The `GeneticEngineBuilder` struct is a builder pattern that allows you to set the parameters of
/// the [GeneticEngine] in a fluent and functional way.
///
/// # Type Parameters
/// - `C`: The type of chromosome used in the genotype, which must implement the [Chromosome] trait.
/// - `T`: The type of the best individual in the population.
pub struct GeneticEngineBuilder<C, T>
where
    C: Chromosome + 'static,
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
    pub(self) fn add_error_if<F>(&mut self, condition: F, message: &str)
    where
        F: Fn() -> bool,
    {
        if condition() {
            self.errors.push(radiate_err!(Builder: "{}", message));
        }
    }

    /// The [ReplacementStrategy] is used to determine how a new individual is added to the [Population]
    /// if an individual is deemed to be either invalid or reaches the maximum age.
    ///
    /// Default is [EncodeReplace], which means that a new individual will be created
    /// be using the `Codec` to encode a new individual from scratch.
    pub fn replace_strategy<R: ReplacementStrategy<C> + 'static>(mut self, replace: R) -> Self {
        self.params.replacement_strategy = Arc::new(replace);
        self
    }

    /// Subscribe to engine events with the given event handler.
    /// The event handler will be called whenever an event is emitted by the engine.
    /// You can use this to log events, or to perform custom actions
    /// based on the events emitted by the engine.
    pub fn subscribe<H>(mut self, handler: H) -> Self
    where
        H: EventHandler<T> + 'static,
    {
        self.params.handlers.push(Arc::new(Mutex::new(handler)));
        self
    }

    /// Set the generation for the engine. This is typically used
    /// when resuming a previously paused or stopped engine.
    pub fn generation(mut self, generation: Generation<C, T>) -> Self {
        self.params.generation = Some(generation);
        self
    }

    pub fn register_metrics(mut self, exprs: Vec<impl Into<MetricQuery>>) -> Self {
        self.params.exprs = Some(Arc::new(Mutex::new(
            exprs.into_iter().map(|e| e.into()).collect(),
        )));
        self
    }

    /// Load a checkpoint from the given file path. This will
    /// load the generation from the file and set it as the current generation
    /// for the engine.
    #[cfg(feature = "serde")]
    pub fn load_checkpoint<P: AsRef<std::path::Path>>(
        mut self,
        path: P,
        reader: impl FileReader<Generation<C, T>>,
    ) -> Self
    where
        C: for<'de> Deserialize<'de>,
        T: for<'de> Deserialize<'de>,
    {
        let read_generation = reader.read(path.as_ref().to_path_buf());
        if let Err(e) = &read_generation {
            self.add_error_if(|| true, &format!("Failed to read checkpoint: {}", e));
        }
        let generation = read_generation.expect("Failed to read checkpoint file");
        self.generation(generation)
    }
}

/// Static step builder for the genetic engine.
impl<C, T> GeneticEngineBuilder<C, T>
where
    C: Chromosome + Clone + PartialEq + 'static,
    T: Clone + Send + Sync + 'static,
{
    /// Build the genetic engine with the given parameters. This will create a new
    /// instance of the [GeneticEngine] with the given parameters.
    pub fn build(self) -> GeneticEngine<C, T> {
        match self.try_build() {
            Ok(engine) => engine,
            Err(e) => panic!("{e}"),
        }
    }

    pub fn try_build(mut self) -> Result<GeneticEngine<C, T>> {
        if !self.errors.is_empty() {
            return Err(radiate_err!(
                Builder: "Failed to build GeneticEngine: {:?}",
                self.errors
            ));
        }

        self.build_problem()?;
        self.build_population()?;
        self.build_alterer()?;
        self.build_front()?;

        let config = EngineConfig::<C, T>::from(&self.params);

        let mut pipeline = Pipeline::<C>::default();

        pipeline.add_step(Self::build_eval_step(&config));
        pipeline.add_step(Self::build_recombine_step(&config));
        pipeline.add_step(Self::build_filter_step(&config));
        pipeline.add_step(Self::build_eval_step(&config));
        pipeline.add_step(Self::build_front_step(&config));
        pipeline.add_step(Self::build_species_step(&config));
        pipeline.add_step(Self::build_audit_step(&config));

        let event_bus = EventBus::new(config.bus_executor(), config.handlers());
        let context = EvolutionContext::from(config);

        Ok(GeneticEngine::<C, T>::new(context, pipeline, event_bus))
    }

    /// Build the problem of the genetic engine. This will create a new problem
    /// using the codec and fitness function if the problem is not set. If the
    /// problem is already set, this function will do nothing. Else, if the fitness function is
    /// a batch fitness function, it will create a new [BatchEngineProblem] and swap the evaluator
    /// to use a [BatchFitnessEvaluator].
    fn build_problem(&mut self) -> Result<()> {
        if self.params.problem_params.problem.is_some() {
            return Ok(());
        }

        ensure!(
            self.params.problem_params.codec.is_some(),
            Builder: "Codec not set"
        );

        let raw_fitness_fn = self.params.problem_params.raw_fitness_fn.clone();
        let fitness_fn = self.params.problem_params.fitness_fn.clone();
        let batch_fitness_fn = self.params.problem_params.batch_fitness_fn.clone();
        let raw_batch_fitness_fn = self.params.problem_params.raw_batch_fitness_fn.clone();

        if batch_fitness_fn.is_some() || raw_batch_fitness_fn.is_some() {
            self.params.problem_params.problem = Some(Arc::new(BatchEngineProblem {
                objective: self.params.optimization_params.objectives.clone(),
                codec: self.params.problem_params.codec.clone().unwrap(),
                batch_fitness_fn,
                raw_batch_fitness_fn,
            }));

            // Replace the evaluator with BatchFitnessEvaluator
            self.params.evaluation_params.evaluator = Arc::new(BatchFitnessEvaluator::new(
                self.params.evaluation_params.fitness_executor.clone(),
            ));

            Ok(())
        } else if fitness_fn.is_some() || raw_fitness_fn.is_some() {
            self.params.problem_params.problem = Some(Arc::new(EngineProblem {
                objective: self.params.optimization_params.objectives.clone(),
                codec: self.params.problem_params.codec.clone().unwrap(),
                fitness_fn,
                raw_fitness_fn,
            }));

            Ok(())
        } else {
            Err(radiate_err!(Builder: "Fitness function not set"))
        }
    }

    /// Build the population of the genetic engine. This will create a new population
    /// using the codec if the population is not set.
    fn build_population(&mut self) -> Result<()> {
        if self.params.population_params.ecosystem.is_some() {
            return Ok(());
        }

        let ecosystem = match &self.params.population_params.ecosystem {
            None => Some(match self.params.problem_params.problem.as_ref() {
                Some(problem) => {
                    let size = self.params.population_params.population_size;
                    let mut phenotypes = Vec::with_capacity(size);

                    for _ in 0..size {
                        let genotype = problem.encode();

                        if !genotype.is_valid() {
                            return Err(radiate_err!(
                                Builder: "Encoded genotype is not valid",
                            ));
                        }

                        phenotypes.push(Phenotype::from((genotype, 0)));
                    }

                    Ecosystem::from(phenotypes)
                }
                None => return Err(radiate_err!(Builder: "Codec not set")),
            }),
            Some(ecosystem) => Some(ecosystem.clone()),
        };

        if let Some(ecosystem) = ecosystem {
            self.params.population_params.ecosystem = Some(ecosystem);
        }

        Ok(())
    }

    /// Build the alterer of the genetic engine. This will create a
    /// new `UniformCrossover` and `UniformMutator` if the alterer is not set.
    /// with a 0.5 crossover rate and a 0.1 mutation rate.
    fn build_alterer(&mut self) -> Result<()> {
        if !self.params.alterers.is_empty() {
            for alter in self.params.alterers.iter_mut() {
                if !alter.rate().is_valid() {
                    return Err(radiate_err!(
                        Builder: "Alterer {} is not valid. Ensure rate {:?} is valid.", alter.name(), alter.rate()
                    ));
                }
            }

            return Ok(());
        }

        let crossover = UniformCrossover::new(0.5).alterer();
        let mutator = UniformMutator::new(0.1).alterer();

        self.params.alterers.push(crossover);
        self.params.alterers.push(mutator);

        Ok(())
    }

    /// Build the pareto front of the genetic engine. This will create a new `Front`
    /// if the front is not set. The `Front` is used to store the best individuals
    /// in the population and is used for multi-objective optimization problems.
    fn build_front(&mut self) -> Result<()> {
        if self.params.optimization_params.front.is_some() {
            return Ok(());
        } else if let Some(generation) = &self.params.generation
            && let Some(front) = generation.front()
        {
            self.params.optimization_params.front = Some(front.clone());
            return Ok(());
        }

        let front_obj = self.params.optimization_params.objectives.clone();
        self.params.optimization_params.front = Some(Front::new(
            self.params.optimization_params.front_range.clone(),
            front_obj,
        ));

        Ok(())
    }

    fn build_eval_step(config: &EngineConfig<C, T>) -> Option<Box<dyn EngineStep<C>>> {
        let eval_step = EvaluateStep {
            objective: config.objective(),
            problem: config.problem(),
            evaluator: config.evaluator(),
        };

        Some(Box::new(eval_step))
    }

    fn build_recombine_step(config: &EngineConfig<C, T>) -> Option<Box<dyn EngineStep<C>>> {
        let offspring_selector = config.offspring_selector();
        let survivor_selector = config.survivor_selector();

        let off_name = offspring_selector.name();
        let offspring_base_name = radiate_utils::intern!(off_name);
        let offspring_time_name = radiate_utils::intern!(format!("{}.time", offspring_base_name));

        let surv_name = survivor_selector.name();
        let survivor_base_name = radiate_utils::intern!(surv_name);
        let survivor_time_name = radiate_utils::intern!(format!("{}.time", survivor_base_name));

        let survivor_select = SelectConfig {
            selector: survivor_selector,
            count: config.survivor_count(),
            names: (survivor_base_name, survivor_time_name),
        };

        let offspring_select = SelectConfig {
            selector: offspring_selector,
            count: config.offspring_count(),
            names: (offspring_base_name, offspring_time_name),
        };

        let recombine_step = RecombineStep {
            survivor: crate::steps::SurvivorConfig {
                select: survivor_select,
            },
            offspring: crate::steps::OffspringConfig {
                select: offspring_select,
                alters: config.alters().to_vec(),
            },
            objective: config.objective(),
            survivor_counts: VersionedCounts::new(),
            offspring_counts: VersionedCounts::new(),
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
        Some(Box::new(MetricStep::new(
            config.objective().clone(),
            config.exprs().clone(),
        )))
    }

    fn build_front_step(config: &EngineConfig<C, T>) -> Option<Box<dyn EngineStep<C>>> {
        if config.objective().is_single() {
            return None;
        }

        let front_step = FrontStep {
            front: config.front(),
        };

        Some(Box::new(front_step))
    }

    fn build_species_step(config: &EngineConfig<C, T>) -> Option<Box<dyn EngineStep<C>>> {
        let diversity = config.diversity()?;

        let species_step = SpeciateStep {
            threshold: config.species_threshold(),
            distance: diversity,
            executor: config.species_executor(),
            objective: config.objective(),
            distances: Vec::new(),
            assignments: Arc::new(Mutex::new(Vec::new())),
        };

        Some(Box::new(species_step))
    }
}

impl<C, T> Default for GeneticEngineBuilder<C, T>
where
    C: Chromosome + 'static,
    T: Clone + Send + 'static,
{
    fn default() -> Self {
        GeneticEngineBuilder {
            params: EngineParams {
                population_params: PopulationParams {
                    population_size: 100,
                    max_age: 20,
                    ecosystem: None,
                },
                species_params: SpeciesParams {
                    diversity: None,
                    species_threshold: Rate::Fixed(0.5),
                    max_species_age: 25,
                    target_species_count: None,
                },
                evaluation_params: EvaluationParams {
                    evaluator: Arc::new(FitnessEvaluator::default()),
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
                problem_params: ProblemParams {
                    codec: None,
                    problem: None,
                    fitness_fn: None,
                    batch_fitness_fn: None,
                    raw_fitness_fn: None,
                    raw_batch_fitness_fn: None,
                },

                replacement_strategy: Arc::new(EncodeReplace),
                alterers: Vec::new(),
                handlers: Vec::new(),
                exprs: None,
                generation: None,
            },
            errors: Vec::new(),
        }
    }
}
