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
use super::{Roulette, Select, ThreadPool, Tournament};

pub struct GeneticEngineParams<'a, G, A, T>
where
    G: Gene<G, A> + Send,
    A: Send,
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
    pub codex: Option<Arc<&'a (dyn Codex<G, A, T> + Send + Sync)>>,
    pub fitness_fn: Option<Arc<dyn Fn(T) -> Score + Send + Sync>>,
}

impl<'a, G, A, T> GeneticEngineParams<'a, G, A, T>
where
    G: Gene<G, A> + Send,
    A: Send,
    T: Clone + Send,
{
    pub fn new() -> Self {
        GeneticEngineParams {
            population_size: 100,
            max_age: 25,
            offspring_fraction: 0.8,
            thread_pool: ThreadPool::new(1),
            optimize: Optimize::Maximize,
            survivor_selector: Box::new(Tournament::new(3)),
            offspring_selector: Box::new(Roulette::new()),
            alterer: None,
            codex: None,
            population: None,
            fitness_fn: None,
        }
    }

    pub fn population_size(mut self, population_size: usize) -> Self {
        self.population_size = population_size;
        self
    }

    pub fn max_age(mut self, max_age: i32) -> Self {
        self.max_age = max_age;
        self
    }

    pub fn offspring_fraction(mut self, offspring_fraction: f32) -> Self {
        self.offspring_fraction = offspring_fraction;
        self
    }

    pub fn codex(mut self, codex: &'a (impl Codex<G, A, T> + Send + Sync)) -> Self {
        self.codex = Some(Arc::new(codex));
        self
    }

    pub fn population(mut self, population: Population<G, A>) -> Self {
        self.population = Some(population);
        self
    }

    pub fn fitness_fn(mut self, fitness_func: impl Fn(T) -> Score + Send + Sync + 'static) -> Self {
        self.fitness_fn = Some(Arc::new(fitness_func));
        self
    }

    pub fn survivor_selector<S: Select<G, A> + 'static>(mut self, selector: S) -> Self {
        self.survivor_selector = Box::new(selector);
        self
    }

    pub fn offspring_selector<S: Select<G, A> + 'static>(mut self, selector: S) -> Self {
        self.offspring_selector = Box::new(selector);
        self
    }

    pub fn alterer(mut self, alterers: Vec<Alterer<G, A>>) -> Self {
        self.alterer = Some(CompositeAlterer::new(alterers));
        self
    }

    pub fn minimizing(mut self) -> Self {
        self.optimize = Optimize::Minimize;
        self
    }

    pub fn maximizing(mut self) -> Self {
        self.optimize = Optimize::Maximize;
        self
    }

    pub fn num_threads(mut self, num_threads: usize) -> Self {
        self.thread_pool = ThreadPool::new(num_threads);
        self
    }

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

    fn build_alterer(&mut self) {
        if self.alterer.is_none() {
            self.alterer = Some(CompositeAlterer::new(vec![
                Alterer::Mutator(0.001),
                Alterer::UniformCrossover(0.5),
            ]));
        }
    }
}
