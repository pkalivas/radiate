use std::sync::Arc;

use crate::engines::alterers::composite_alterer::CompositeAlterer;
use crate::engines::genetic_engine::GeneticEngine;
use crate::engines::genome::genes::gene::Gene;
use crate::engines::genome::phenotype::Phenotype;
use crate::engines::genome::population::Population;
use crate::engines::optimize::Optimize;
use crate::engines::score::Score;

use super::alterers::alter::Alterer;
use super::codexes::Codex;
use super::{RouletteSelector, Select, ThreadPool, TournamentSelector};

/// Parameters for the genetic engine.
/// This struct is used to configure the genetic engine before it is created.
/// 
/// When the ```GeneticEngineParams```  calls the ```build``` method, it will create a new instance
/// of the ```GeneticEngine``` with the given parameters. If any of the required parameters are not
/// set, the ```build``` method will panic. At a minimum, the ```codex``` and ```fitness_fn``` must be set.
/// The ```GeneticEngineParams``` struct is a builder pattern that allows you to set the parameters of 
/// the ```GeneticEngine``` in a fluent and functional way.
pub struct GeneticEngineParams<'a, G, A, T>
where
    G: Gene<G, A>,
    T: Clone,
{
    pub population_size: usize,
    pub max_age: i32,
    pub offspring_fraction: f32,
    pub thread_pool: ThreadPool,
    pub optimize: Optimize,
    pub survivor_selector: Box<dyn Select<G, A>>,
    pub offspring_selector: Box<dyn Select<G, A>>,
    pub alterer: Option<CompositeAlterer<G, A>>,
    pub population: Option<Population<G, A>>,
    pub codex: Option<Arc<&'a dyn Codex<G, A, T>>>,
    pub fitness_fn: Option<Arc<dyn Fn(T) -> Score + Send + Sync>>,
}

impl<'a, G, A, T> GeneticEngineParams<'a, G, A, T>
where
    G: Gene<G, A>,
    T: Clone + Send,
{
    /// Create a new instance of the GeneticEngineParams. This will create a new instance with the following defaults:
    /// * population_size: 100
    /// * max_age: 25
    /// * offspring_fraction: 0.8
    ///     * This is a value from 0..=1 that represents the fraction of 
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
        Self {
            population_size: 100,
            max_age: 25,
            offspring_fraction: 0.8,
            thread_pool: ThreadPool::new(1),
            optimize: Optimize::Maximize,
            survivor_selector: Box::new(TournamentSelector::new(3)),
            offspring_selector: Box::new(RouletteSelector::new()),
            alterer: None,
            codex: None,
            population: None,
            fitness_fn: None,
        }
    }

    /// Set the population size of the genetic engine. Default is 100.
    pub fn population_size(mut self, population_size: usize) -> Self {
        self.population_size = population_size;
        self
    }

    /// Set the maximum age of an individual in the population. Default is 25.
    pub fn max_age(mut self, max_age: i32) -> Self {
        self.max_age = max_age;
        self
    }

    /// Set the fraction of the population that will be replaced by offspring each generation.
    /// Default is 0.8. This is a value from 0..=1 that represents the fraction of
    /// population that will be replaced by offspring each generation. The remainder will 'survive' to the next generation.
    pub fn offspring_fraction(mut self, offspring_fraction: f32) -> Self {
        self.offspring_fraction = offspring_fraction;
        self
    }

    /// Set the codex that will be used to encode and decode the genotype of the population.
    pub fn codex(mut self, codex: &'a impl Codex<G, A, T>) -> Self {
        self.codex = Some(Arc::new(codex));
        self
    }

    /// Set the population of the genetic engine. This is useful if you want to provide a custom population.
    /// If this is not set, the genetic engine will create a new population of ```population_size``` using the codex.
    pub fn population(mut self, population: Population<G, A>) -> Self {
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
    pub fn survivor_selector<S: Select<G, A> + 'static>(mut self, selector: S) -> Self {
        self.survivor_selector = Box::new(selector);
        self
    }

    /// Set the offspring selector of the genetic engine. This is the selector that will be used to select the offspring of the population.
    /// Default is RouletteSelector.
    pub fn offspring_selector<S: Select<G, A> + 'static>(mut self, selector: S) -> Self {
        self.offspring_selector = Box::new(selector);
        self
    }

    /// Set the alterer of the genetic engine. This is the alterer that will be used to alter the offspring of the population.
    /// The alterer is used to apply mutations and crossover operations to the offspring and will be used to create the next generation of the population.
    /// Note, the order of the alterers is important. The alterers will be applied in the order they are provided.
    pub fn alterer(mut self, alterers: Vec<Alterer<G, A>>) -> Self {
        self.alterer = Some(CompositeAlterer::new(alterers));
        self
    }

    /// Set the optimization goal of the genetic engine to minimize the fitness function.
    pub fn minimizing(mut self) -> Self {
        self.optimize = Optimize::Minimize;
        self
    }

    /// Set the optimization goal of the genetic engine to maximize the fitness function.
    pub fn maximizing(mut self) -> Self {
        self.optimize = Optimize::Maximize;
        self
    }

    /// Set the thread pool of the genetic engine. This is the thread pool that will be used to execute the fitness function in parallel.
    /// Some fitness functions may be computationally expensive and can benefit from parallel execution.
    pub fn num_threads(mut self, num_threads: usize) -> Self {
        self.thread_pool = ThreadPool::new(num_threads);
        self
    }

    /// Build the genetic engine with the given parameters. This will create a new instance of the ```GeneticEngine``` with the given parameters.
    pub fn build(mut self) -> GeneticEngine<'a, G, A, T> {
        self.build_population();
        self.build_alterer();

        if !self.codex.is_some() {
            panic!("Codex not set");
        }

        if !self.fitness_fn.is_some() {
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
        if self.alterer.is_none() {
            self.alterer = Some(CompositeAlterer::new(vec![
                Alterer::Mutator(0.001),
                Alterer::UniformCrossover(0.5),
            ]));
        }
    }
}
