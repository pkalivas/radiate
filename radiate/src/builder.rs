use super::codexes::Codex;
use super::thread_pool::ThreadPool;
use super::{
    Alter, AlterAction, Audit, Crossover, EncodeReplace, EngineProblem, Front, GeneticEngineParams,
    MetricAudit, Mutate, Problem, ReplacementStrategy, RouletteSelector, Select,
    TournamentSelector, pareto,
};
use crate::Chromosome;
use crate::engine::GeneticEngine;
use crate::genome::phenotype::Phenotype;
use crate::genome::population::Population;
use crate::objectives::Score;
use crate::objectives::{Objective, Optimize};
use crate::uniform::{UniformCrossover, UniformMutator};
use std::cmp::Ordering;
use std::ops::Range;
use std::sync::Arc;

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
pub struct GeneticEngineBuilder<C, T>
where
    C: Chromosome + 'static,
    T: Clone + 'static,
{
    pub population_size: usize,
    pub max_age: usize,
    pub front_range: Range<usize>,
    pub offspring_fraction: f32,
    pub thread_pool: ThreadPool,
    pub objective: Objective,
    pub survivor_selector: Arc<dyn Select<C>>,
    pub offspring_selector: Arc<dyn Select<C>>,
    pub alterers: Vec<Arc<dyn Alter<C>>>,
    pub audits: Vec<Arc<dyn Audit<C>>>,
    pub population: Option<Population<C>>,
    pub codex: Option<Arc<dyn Codex<C, T>>>,
    pub fitness_fn: Option<Arc<dyn Fn(T) -> Score + Send + Sync>>,
    pub problem: Option<Arc<dyn Problem<C, T>>>,
    pub replacement_strategy: Arc<dyn ReplacementStrategy<C>>,
    pub front: Option<Front<Phenotype<C>>>,
}

impl<C, T> GeneticEngineBuilder<C, T>
where
    C: Chromosome,
    T: Clone + Send,
{
    /// Set the population size of the genetic engine. Default is 100.
    pub fn population_size(mut self, population_size: usize) -> Self {
        if population_size < 1 {
            panic!("population_size must be greater than 0");
        }

        self.population_size = population_size;
        self
    }

    /// Set the maximum age of an individual in the population. Default is 25.
    pub fn max_age(mut self, max_age: usize) -> Self {
        if max_age < 1 {
            panic!("max_age must be greater than 0");
        }

        self.max_age = max_age;
        self
    }

    /// The `FilterStrategy` is used to determine how a new individual is added to the `Population`
    /// if an individual is deemed to be either invalid or reaches the maximum age.
    ///
    /// Default is `FilterStrategy::Encode`, which means that a new individual will be created
    /// be using the `Codex` to encode a new individual from scratch.
    pub fn replace_strategy<R: ReplacementStrategy<C> + 'static>(mut self, replace: R) -> Self {
        self.replacement_strategy = Arc::new(replace);
        self
    }

    /// Add a single audit to the algorithm that will produce additional metrics
    /// to collect during the evolution process.
    pub fn audit(mut self, audit: impl Audit<C> + 'static) -> Self {
        self.audits.push(Arc::new(audit));
        self
    }

    /// Add a list of audits to the algorithm that will produce additional metrics
    /// to collect during the evolution process.
    pub fn audits(mut self, audits: Vec<Arc<dyn Audit<C>>>) -> Self {
        self.audits.extend(audits);
        self
    }

    /// Set the fraction of the population that will be replaced by offspring each generation.
    /// Default is 0.8. This is a value from 0...=1 that represents the fraction of
    /// population that will be replaced by offspring each generation. The remainder will 'survive' to the next generation.
    pub fn offspring_fraction(mut self, offspring_fraction: f32) -> Self {
        if !(0.0..=1.0).contains(&offspring_fraction) {
            panic!("offspring_fraction must be between 0 and 1");
        }

        self.offspring_fraction = offspring_fraction;
        self
    }

    /// Set the codex that will be used to encode and decode the genotype of the population.
    pub fn codex<D: Codex<C, T> + 'static>(mut self, codex: D) -> Self {
        self.codex = Some(Arc::new(codex));
        self
    }

    /// Set the problem of the genetic engine. This is useful if you want to provide a custom problem.
    pub fn problem<P: Problem<C, T> + 'static>(mut self, problem: P) -> Self {
        self.problem = Some(Arc::new(problem));
        self
    }

    /// Set the population of the genetic engine. This is useful if you want to provide a custom population.
    /// If this is not set, the genetic engine will create a new population of `population_size` using the codex.
    pub fn population(mut self, population: Population<C>) -> Self {
        self.population = Some(population);
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
        self.fitness_fn = Some(Arc::new(other));
        self
    }

    /// Set the survivor selector of the genetic engine. This is the selector that will
    /// be used to select the survivors of the population. Default is `TournamentSelector`
    /// with a group size of 3.
    pub fn survivor_selector<S: Select<C> + 'static>(mut self, selector: S) -> Self {
        self.survivor_selector = Arc::new(selector);
        self
    }

    /// Set the offspring selector of the genetic engine. This is the selector that will
    /// be used to select the offspring of the population. Default is `RouletteSelector`.
    pub fn offspring_selector<S: Select<C> + 'static>(mut self, selector: S) -> Self {
        self.offspring_selector = Arc::new(selector);
        self
    }

    /// Set the alterer of the genetic engine. This is the alterer that will be used to
    /// alter the offspring of the population. The alterer is used to apply mutations
    /// and crossover operations to the offspring and will be used to create the next
    /// generation of the population. **Note**: the order of the alterers is important - the
    /// alterers will be applied in the order they are provided.
    pub fn alter(mut self, alterers: Vec<Box<dyn Alter<C>>>) -> Self {
        self.alterers = alterers.into_iter().map(|alt| alt.into()).collect();
        self
    }

    /// Define a single mutator for the genetic engine - this will be  converted to
    /// a `Box<dyn Alter<C>>` and added to the list of alterers. Note: The order in which
    /// mutators and crossovers are added is the order in which they will be applied during
    /// the evolution process.
    pub fn mutator<M: Mutate<C> + 'static>(mut self, mutator: M) -> Self {
        self.alterers.push(Arc::new(mutator.alterer()));
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

        self.alterers.extend(mutate_actions);
        self
    }

    /// Define a single crossover for the genetic engine - this will be converted to
    /// a `Box<dyn Alter<C>>` and added to the list of alterers. Note: The order in which
    /// mutators and crossovers are added is the order in which they will be applied during
    /// the evolution process.s
    pub fn crossover<R: Crossover<C> + 'static>(mut self, crossover: R) -> Self {
        self.alterers.push(Arc::new(crossover.alterer()));
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

        self.alterers.extend(crossover_actions);
        self
    }

    /// Set the optimization goal of the genetic engine to minimize the fitness function.
    pub fn minimizing(mut self) -> Self {
        self.objective = Objective::Single(Optimize::Minimize);
        self
    }

    /// Set the optimization goal of the genetic engine to maximize the fitness function.
    pub fn maximizing(mut self) -> Self {
        self.objective = Objective::Single(Optimize::Maximize);
        self
    }

    pub fn multi_objective(mut self, objectives: Vec<Optimize>) -> Self {
        self.objective = Objective::Multi(objectives);
        self
    }

    /// Set the minimum and maximum size of the pareto front. This is used for
    /// multi-objective optimization problems where the goal is to find the best
    /// solutions that are not dominated by any other solution.
    pub fn front_size(mut self, range: Range<usize>) -> Self {
        self.front_range = range;
        self
    }

    /// Set the thread pool of the genetic engine. This is the thread pool that will be used
    /// to execute the fitness function in parallel. Some fitness functions may be computationally
    /// expensive and can benefit from parallel execution.
    pub fn num_threads(mut self, num_threads: usize) -> Self {
        self.thread_pool = ThreadPool::new(num_threads);
        self
    }

    /// Build the genetic engine with the given parameters. This will create a new
    /// instance of the `GeneticEngine` with the given parameters.
    pub fn build(mut self) -> GeneticEngine<C, T> {
        if self.problem.is_none() {
            if self.codex.is_none() {
                panic!("Codex not set");
            }

            if self.fitness_fn.is_none() {
                panic!("Fitness function not set");
            }

            let problem = EngineProblem {
                codex: self.codex.clone().unwrap(),
                fitness_fn: self.fitness_fn.clone().unwrap(),
            };

            self.problem(problem).build()
        } else {
            self.build_population();
            self.build_alterer();
            self.build_front();

            GeneticEngine::new(GeneticEngineParams::new(
                self.population.unwrap(),
                self.problem.unwrap(),
                self.survivor_selector,
                self.offspring_selector,
                self.replacement_strategy,
                self.audits,
                self.alterers,
                self.objective,
                self.thread_pool,
                self.max_age,
                self.front.clone().unwrap(),
                self.offspring_fraction,
            ))
        }
    }

    /// Build the population of the genetic engine. This will create a new population
    /// using the codex if the population is not set.
    fn build_population(&mut self) {
        self.population = match &self.population {
            None => Some(match self.problem.as_ref() {
                Some(problem) => Population::from((self.population_size, || {
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
        if !self.alterers.is_empty() {
            return;
        }

        let crossover = Arc::new(UniformCrossover::new(0.5).alterer()) as Arc<dyn Alter<C>>;
        let mutator = Arc::new(UniformMutator::new(0.1).alterer()) as Arc<dyn Alter<C>>;

        self.alterers.push(crossover);
        self.alterers.push(mutator);
    }

    /// Build the pareto front of the genetic engine. This will create a new `Front`
    /// if the front is not set. The `Front` is used to store the best individuals
    /// in the population and is used for multi-objective optimization problems.
    fn build_front(&mut self) {
        if self.front.is_some() {
            return;
        }

        let front_obj = self.objective.clone();
        self.front = Some(Front::new(
            self.front_range.clone(),
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
    C: Chromosome + 'static,
    T: Clone + Send + 'static,
{
    fn default() -> Self {
        GeneticEngineBuilder {
            population_size: 100,
            max_age: 20,
            offspring_fraction: 0.8,
            front_range: 800..900,
            thread_pool: ThreadPool::new(1),
            objective: Objective::Single(Optimize::Maximize),
            survivor_selector: Arc::new(TournamentSelector::new(3)),
            offspring_selector: Arc::new(RouletteSelector::new()),
            replacement_strategy: Arc::new(EncodeReplace),
            audits: vec![Arc::new(MetricAudit)],
            alterers: Vec::new(),
            codex: None,
            population: None,
            fitness_fn: None,
            problem: None,
            front: None,
        }
    }
}
