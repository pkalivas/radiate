use crate::Chromosome;
use crate::Generation;
use crate::GeneticEngineBuilder;
use crate::FrozenMap;
use crate::builder::EngineParams;
use crate::builder::evaluators::EvaluationParams;
use crate::builder::objectives::OptimizeParams;
use crate::builder::population::PopulationParams;
use crate::builder::problem::ProblemParams;
use crate::builder::selectors::SelectionParams;
use crate::builder::species::SpeciesParams;
use crate::genome::phenotype::Phenotype;
use crate::objectives::Objective;
use crate::{EventHandler, Front, Problem, ReplacementStrategy, Select};
use radiate_core::NamedExpr;
use radiate_core::{Alterer, Diversity, Ecosystem, Evaluator, Executor, Genotype, Lineage, Rate};
use std::sync::{Arc, Mutex, RwLock};

#[derive(Clone)]
pub(crate) struct EngineConfig<C: Chromosome, T: Clone> {
    ecosystem: Ecosystem<C>,
    problem: Arc<dyn Problem<C, T>>,
    survivor_selector: Arc<dyn Select<C>>,
    offspring_selector: Arc<dyn Select<C>>,
    replacement_strategy: Arc<dyn ReplacementStrategy<C>>,
    alterers: Vec<Alterer<C>>,
    species_threshold: Rate,
    diversity: Option<Arc<dyn Diversity<C>>>,
    evaluator: Arc<dyn Evaluator<C, T>>,
    objective: Objective,
    max_age: usize,
    max_species_age: usize,
    front: Arc<RwLock<Front<Phenotype<C>>>>,
    lineage: Arc<RwLock<Lineage>>,
    offspring_fraction: f32,
    executor: EvaluationParams<C, T>,
    handlers: Vec<Arc<Mutex<dyn EventHandler<T>>>>,
    exprs: Option<Arc<Mutex<Vec<NamedExpr>>>>,
    generation: Option<Generation<C, T>>,
    freeze: FrozenMap,
}

impl<C: Chromosome, T: Clone> EngineConfig<C, T> {
    pub fn ecosystem(&self) -> &Ecosystem<C> {
        &self.ecosystem
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

    pub fn alters(&self) -> &[Alterer<C>] {
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

    pub fn species_threshold(&self) -> Rate {
        self.species_threshold.clone()
    }

    pub fn diversity(&self) -> Option<Arc<dyn Diversity<C>>> {
        self.diversity.clone()
    }

    pub fn front(&self) -> Arc<RwLock<Front<Phenotype<C>>>> {
        Arc::clone(&self.front)
    }

    pub fn lineage(&self) -> Arc<RwLock<Lineage>> {
        Arc::clone(&self.lineage)
    }

    pub fn evaluator(&self) -> Arc<dyn Evaluator<C, T>> {
        Arc::clone(&self.evaluator)
    }

    pub fn survivor_count(&self) -> usize {
        self.ecosystem.population().len() - self.offspring_count()
    }

    pub fn offspring_count(&self) -> usize {
        (self.ecosystem.population().len() as f32 * self.offspring_fraction) as usize
    }

    pub fn bus_executor(&self) -> Arc<Executor> {
        Arc::clone(&self.executor.bus_executor)
    }

    pub fn species_executor(&self) -> Arc<Executor> {
        Arc::clone(&self.executor.species_executor)
    }

    pub fn handlers(&self) -> Vec<Arc<Mutex<dyn EventHandler<T>>>> {
        self.handlers.clone()
    }

    pub fn problem(&self) -> Arc<dyn Problem<C, T>> {
        Arc::clone(&self.problem)
    }

    pub fn generation(&self) -> Option<Generation<C, T>>
    where
        C: Clone,
        T: Clone,
    {
        self.generation.clone()
    }

    pub fn encoder(&self) -> Arc<dyn Fn() -> Genotype<C> + Send + Sync>
    where
        C: 'static,
        T: 'static,
    {
        let problem = Arc::clone(&self.problem);
        Arc::new(move || problem.encode())
    }

    pub fn exprs(&self) -> Option<Arc<Mutex<Vec<NamedExpr>>>> {
        self.exprs.clone()
    }

    pub fn freeze(&self) -> FrozenMap {
        self.freeze.clone()
    }
}

impl<C, T> From<&EngineParams<C, T>> for EngineConfig<C, T>
where
    C: Chromosome + Clone + 'static,
    T: Clone + Send + Sync + 'static,
{
    fn from(params: &EngineParams<C, T>) -> Self {
        Self {
            ecosystem: params.population_params.ecosystem.clone().unwrap(),
            problem: params.problem_params.problem.clone().unwrap(),
            survivor_selector: params.selection_params.survivor_selector.clone(),
            offspring_selector: params.selection_params.offspring_selector.clone(),
            replacement_strategy: params.replacement_strategy.clone(),
            alterers: params.alterers.clone(),
            objective: params.optimization_params.objectives.clone(),
            max_age: params.population_params.max_age,
            max_species_age: params.species_params.max_species_age,
            species_threshold: params.species_params.species_threshold.clone(),
            diversity: params.species_params.diversity.clone(),
            front: Arc::new(RwLock::new(
                params.optimization_params.front.clone().unwrap(),
            )),
            lineage: Arc::new(RwLock::new(Lineage::default())),
            offspring_fraction: params.selection_params.offspring_fraction,
            evaluator: params.evaluation_params.evaluator.clone(),
            executor: params.evaluation_params.clone(),
            handlers: params.handlers.clone(),
            generation: params.generation.clone(),
            exprs: params.exprs.clone(),
            freeze: params.freeze.clone(),
        }
    }
}

impl<C, T> From<EngineConfig<C, T>> for GeneticEngineBuilder<C, T>
where
    C: Chromosome + Clone + 'static,
    T: Clone + Send + Sync + 'static,
{
    fn from(config: EngineConfig<C, T>) -> Self {
        GeneticEngineBuilder {
            params: EngineParams {
                population_params: PopulationParams {
                    population_size: config.ecosystem.population().len(),
                    max_age: config.max_age,
                    ecosystem: Some(config.ecosystem),
                },
                species_params: SpeciesParams {
                    diversity: config.diversity,
                    species_threshold: config.species_threshold,
                    max_species_age: config.max_species_age,
                },
                evaluation_params: config.executor,
                selection_params: SelectionParams {
                    offspring_fraction: config.offspring_fraction,
                    survivor_selector: config.survivor_selector,
                    offspring_selector: config.offspring_selector,
                },
                optimization_params: OptimizeParams {
                    objectives: config.objective,
                    front_range: config.front.read().unwrap().range().clone(),
                    front: Some(config.front.read().unwrap().clone()),
                },
                problem_params: ProblemParams {
                    codec: None,
                    problem: Some(config.problem),
                    fitness_fn: None,
                    batch_fitness_fn: None,
                    raw_fitness_fn: None,
                    raw_batch_fitness_fn: None,
                },

                replacement_strategy: config.replacement_strategy,
                alterers: config.alterers,
                handlers: config.handlers,
                exprs: config.exprs,
                generation: config.generation,
                freeze: config.freeze,
            },
            errors: Vec::new(),
        }
    }
}
