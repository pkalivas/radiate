use super::codexes::Codex;
use super::thread_pool::ThreadPool;
use super::{RouletteSelector, Select, TournamentSelector};
use crate::engines::genetic_engine::GeneticEngine;
use crate::engines::genome::phenotype::Phenotype;
use crate::engines::genome::population::Population;
use crate::engines::score::Score;
use crate::objectives::{Objective, Optimize};
use crate::uniform_crossover::UniformCrossover;
use crate::uniform_mutator::UniformMutator;
use crate::{Alter, Chromosome};
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
pub struct GeneticEngineParams<'a, C, T>
where
    C: Chromosome,
    T: Clone,
{
    pub population_size: usize,
    pub max_age: i32,
    pub min_front_size: usize,
    pub max_front_size: usize,
    pub offspring_fraction: f32,
    pub thread_pool: ThreadPool,
    pub objective: Objective,
    pub survivor_selector: Box<dyn Select<C>>,
    pub offspring_selector: Box<dyn Select<C>>,
    pub alterers: Vec<Box<dyn Alter<C>>>,
    pub population: Option<Population<C>>,
    pub codex: Option<Arc<&'a dyn Codex<C, T>>>,
    pub fitness_fn: Option<Arc<dyn Fn(T) -> Score + Send + Sync>>,
}

impl<'a, C, T> GeneticEngineParams<'a, C, T>
where
    C: Chromosome,
    T: Clone + Send,
{
    /// Create a new instance of the GeneticEngineParams. This will create a new instance with the following defaults:
    /// * population_size: 100
    /// * max_age: 20
    /// * offspring_fraction: 0.8
    ///     * This is a value from 0...=1 that represents the fraction of
    ///       population that will be replaced by offspring each generation.
    ///       For example, if the population size is 100 and the offspring_fraction is 0.8,
    ///       then 80 individuals will be replaced by offspring each generation.
    ///       The remaining 20 individuals will be selected from the survivors of the previous generation.
    /// * thread_pool: ThreadPool::new(1)
    ///     * This is a thread pool that is used to execute the fitness function in parallel. Default is 1 thread.
    /// * optimize: Optimize::Maximize
    ///     * This is the optimization goal of the genetic engine. The default is to maximize the fitness function.
    /// * survivor_selector: TournamentSelector::new(3)
    /// * offspring_selector: RouletteSelector::new()
    pub fn new() -> Self {
        GeneticEngineParams {
            population_size: 100,
            max_age: 20,
            offspring_fraction: 0.8,
            min_front_size: 1000,
            max_front_size: 1500,
            thread_pool: ThreadPool::new(1),
            objective: Objective::Single(Optimize::Maximize),
            survivor_selector: Box::new(TournamentSelector::new(3)),
            offspring_selector: Box::new(RouletteSelector::new()),
            alterers: Vec::new(),
            codex: None,
            population: None,
            fitness_fn: None,
        }
    }

    /// Set the population size of the genetic engine. Default is 100.
    pub fn population_size(mut self, population_size: usize) -> Self {
        if population_size < 1 {
            panic!("population_size must be greater than 0");
        }
        
        self.population_size = population_size;
        self
    }

    /// Set the maximum age of an individual in the population. Default is 25.
    pub fn max_age(mut self, max_age: i32) -> Self {
        if max_age < 1 {
            panic!("max_age must be greater than 0");
        }

        self.max_age = max_age;
        self
    }

    /// Set the fraction of the population that will be replaced by offspring each generation.
    /// Default is 0.8. This is a value from 0...=1 that represents the fraction of
    /// population that will be replaced by offspring each generation. The remainder will 'survive' to the next generation.
    pub fn offspring_fraction(mut self, offspring_fraction: f32) -> Self {
        if offspring_fraction < 0.0 || offspring_fraction > 1.0 {
            panic!("offspring_fraction must be between 0 and 1");
        }

        self.offspring_fraction = offspring_fraction;
        self
    }

    /// Set the codex that will be used to encode and decode the genotype of the population.
    pub fn codex(mut self, codex: &'a impl Codex<C, T>) -> Self {
        self.codex = Some(Arc::new(codex));
        self
    }

    /// Set the population of the genetic engine. This is useful if you want to provide a custom population.
    /// If this is not set, the genetic engine will create a new population of ```population_size``` using the codex.
    pub fn population(mut self, population: Population<C>) -> Self {
        self.population = Some(population);
        self
    }

    /// Set the fitness function of the genetic engine. This is the function that will be used to evaluate the fitness of each individual in the population.
    /// This function should take a single argument of type T and return a Score. The Score is used to evaluate or rank the fitness of the individual.
    /// This method is required and must be set before calling the ```build``` method.
    pub fn fitness_fn(mut self, fitness_func: impl Fn(T) -> Score + Send + Sync + 'static) -> Self {
        self.fitness_fn = Some(Arc::new(fitness_func));
        self
    }

    /// Set the survivor selector of the genetic engine. This is the selector that will be used to select the survivors of the population.
    /// Default is TournamentSelector with a group size of 3.
    pub fn survivor_selector<S: Select<C> + 'static>(mut self, selector: S) -> Self {
        self.survivor_selector = Box::new(selector);
        self
    }

    /// Set the offspring selector of the genetic engine. This is the selector that will be used to select the offspring of the population.
    /// Default is RouletteSelector.
    pub fn offspring_selector<S: Select<C> + 'static>(mut self, selector: S) -> Self {
        self.offspring_selector = Box::new(selector);
        self
    }

    /// Set the alterer of the genetic engine. This is the alterer that will be used to alter the offspring of the population.
    /// The alterer is used to apply mutations and crossover operations to the offspring and will be used to create the next generation of the population.
    /// Note, the order of the alterers is important. The alterers will be applied in the order they are provided.
    // pub fn alterer(mut self, alterers: Vec<Box<dyn Alter<C>>>) -> Self {
    pub fn alter(mut self, alterers: Vec<Box<dyn Alter<C>>>) -> Self {
        self.alterers = alterers;
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

    pub fn front_size(mut self, min_size: usize, max_size: usize) -> Self {
        self.min_front_size = min_size;
        self.max_front_size = max_size;
        self
    }

    /// Set the thread pool of the genetic engine. This is the thread pool that will be used to execute the fitness function in parallel.
    /// Some fitness functions may be computationally expensive and can benefit from parallel execution.
    pub fn num_threads(mut self, num_threads: usize) -> Self {
        self.thread_pool = ThreadPool::new(num_threads);
        self
    }

    /// Build the genetic engine with the given parameters. This will create a new instance of the ```GeneticEngine``` with the given parameters.
    pub fn build(mut self) -> GeneticEngine<'a, C, T> {
        self.build_population();
        self.build_alterer();

        if self.codex.is_none() {
            panic!("Codex not set");
        }

        if self.fitness_fn.is_none() {
            panic!("Fitness function not set");
        }

        GeneticEngine::new(self)
    }

    /// Build the population of the genetic engine. This will create a new population using the codex if the population is not set.
    fn build_population(&mut self) {
        self.population = match &self.population {
            None => Some(match self.codex.as_ref() {
                Some(codex) => Population::from_fn(self.population_size, || {
                    Phenotype::from_genotype(codex.encode(), 0)
                }),
                None => panic!("Codex not set"),
            }),
            Some(pop) => Some(pop.clone()),
        };
    }

    /// Build the alterer of the genetic engine. This will create a new alterer if the alterer is not set.
    fn build_alterer(&mut self) {
        if !self.alterers.is_empty() {
            return;
        }

        let mutator = Box::new(UniformMutator::new(0.001));
        let crossover = Box::new(UniformCrossover::new(0.5));

        self.alterers.push(mutator);
        self.alterers.push(crossover);
    }
}
