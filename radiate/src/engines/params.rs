use super::thread_pool::ThreadPool;
use super::{Alter, EngineError, Front, GeneticEngine, Problem, ReplacementStrategy, Select};
use crate::Chromosome;
use crate::engines::genome::population::Population;
use crate::objectives::Objective;
use std::sync::Arc;

pub struct GeneticEngineParams<C: Chromosome, T>
where
    C: Chromosome,
    T: Clone + Default + Send,
{
    population: Option<Population<C>>,
    problem: Arc<dyn Problem<C, T>>,
    survivor_selector: Box<dyn Select<C>>,
    offspring_selector: Box<dyn Select<C>>,
    replacement_strategy: Box<dyn ReplacementStrategy<C>>,
    alterers: Option<Vec<Box<dyn Alter<C>>>>,
    objective: Objective,
    thread_pool: ThreadPool,
    max_age: usize,
    front: Front,
    offspring_fraction: f32,
    error: Option<EngineError>,
}

impl<C: Chromosome, T> GeneticEngineParams<C, T>
where
    C: Chromosome,
    T: Clone + Default + Send,
{
    pub fn new(
        population: Option<Population<C>>,
        problem: Arc<dyn Problem<C, T>>,
        survivor_selector: Box<dyn Select<C>>,
        offspring_selector: Box<dyn Select<C>>,
        replacement_strategy: Box<dyn ReplacementStrategy<C>>,
        alterers: Option<Vec<Box<dyn Alter<C>>>>,
        objective: Objective,
        thread_pool: ThreadPool,
        max_age: usize,
        front: Front,
        offspring_fraction: f32,
        error: Option<EngineError>,
    ) -> Self {
        Self {
            population,
            problem,
            survivor_selector,
            offspring_selector,
            replacement_strategy,
            alterers,
            objective,
            thread_pool,
            max_age,
            front,
            offspring_fraction,
            error,
        }
    }

    pub fn population(&self) -> Option<&Population<C>> {
        self.population.as_ref()
    }

    pub fn problem(&self) -> Arc<dyn Problem<C, T>> {
        Arc::clone(&self.problem)
    }

    pub fn survivor_selector(&self) -> &Box<dyn Select<C>> {
        &self.survivor_selector
    }

    pub fn offspring_selector(&self) -> &Box<dyn Select<C>> {
        &self.offspring_selector
    }

    pub fn replacement_strategy(&self) -> &Box<dyn ReplacementStrategy<C>> {
        &self.replacement_strategy
    }

    pub fn alters(&self) -> &[Box<dyn Alter<C>>] {
        self.alterers.as_ref().map(|v| v.as_slice()).unwrap_or(&[])
    }

    pub fn objective(&self) -> &Objective {
        &self.objective
    }

    pub fn thread_pool(&self) -> &ThreadPool {
        &self.thread_pool
    }

    pub fn max_age(&self) -> usize {
        self.max_age
    }

    pub fn front(&self) -> &Front {
        &self.front
    }

    pub fn survivor_count(&self) -> usize {
        match self.population() {
            Some(pop) => pop.len() - self.offspring_count(),
            None => 0,
        }
    }

    pub fn offspring_count(&self) -> usize {
        match self.population() {
            Some(pop) => (pop.len() as f32 * self.offspring_fraction) as usize,
            None => 0,
        }
    }

    pub fn errors(&self) -> Option<EngineError> {
        self.error.clone()
    }
}

impl<C, T> Into<GeneticEngine<C, T>> for GeneticEngineParams<C, T>
where
    C: Chromosome,
    T: Clone + Default + Send,
{
    fn into(self) -> GeneticEngine<C, T> {
        GeneticEngine::new(self)
    }
}
