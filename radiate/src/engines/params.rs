use super::thread_pool::ThreadPool;
use super::{Alter, Audit, Distance, Front, Problem, ReplacementStrategy, Select};
use crate::Chromosome;
use crate::engines::genome::phenotype::Phenotype;
use crate::engines::genome::population::Population;
use crate::objectives::Objective;
use std::sync::Arc;

pub struct GeneticEngineParams<C: Chromosome, T> {
    population: Population<C>,
    problem: Arc<dyn Problem<C, T>>,
    survivor_selector: Box<dyn Select<C>>,
    offspring_selector: Box<dyn Select<C>>,
    replacement_strategy: Box<dyn ReplacementStrategy<C>>,
    audits: Vec<Arc<dyn Audit<C>>>,
    distance: Option<Arc<dyn Distance<C>>>,
    alterers: Vec<Box<dyn Alter<C>>>,
    objective: Objective,
    thread_pool: ThreadPool,
    max_age: usize,
    front: Front<Phenotype<C>>,
    offspring_fraction: f32,
}

impl<C: Chromosome, T> GeneticEngineParams<C, T> {
    pub fn new(
        population: Population<C>,
        problem: Arc<dyn Problem<C, T>>,
        survivor_selector: Box<dyn Select<C>>,
        offspring_selector: Box<dyn Select<C>>,
        replacement_strategy: Box<dyn ReplacementStrategy<C>>,
        audits: Vec<Arc<dyn Audit<C>>>,
        distance: Option<Arc<dyn Distance<C>>>,
        alterers: Vec<Box<dyn Alter<C>>>,
        objective: Objective,
        thread_pool: ThreadPool,
        max_age: usize,
        front: Front<Phenotype<C>>,
        offspring_fraction: f32,
    ) -> Self {
        Self {
            population,
            problem,
            survivor_selector,
            offspring_selector,
            replacement_strategy,
            audits,
            distance,
            alterers,
            objective,
            thread_pool,
            max_age,
            front,
            offspring_fraction,
        }
    }

    pub fn population(&self) -> &Population<C> {
        &self.population
    }

    pub fn problem(&self) -> Arc<dyn Problem<C, T>> {
        Arc::clone(&self.problem)
    }

    pub fn survivor_selector(&self) -> &dyn Select<C> {
        &*self.survivor_selector
    }

    pub fn offspring_selector(&self) -> &dyn Select<C> {
        &*self.offspring_selector
    }

    pub fn replacement_strategy(&self) -> &dyn ReplacementStrategy<C> {
        &*self.replacement_strategy
    }

    pub fn audits(&self) -> &[Arc<dyn Audit<C>>] {
        &self.audits
    }

    pub fn distance(&self) -> Option<Arc<dyn Distance<C>>> {
        self.distance.clone()
    }

    pub fn alters(&self) -> &[Box<dyn Alter<C>>] {
        &self.alterers
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

    pub fn front(&self) -> &Front<Phenotype<C>> {
        &self.front
    }

    pub fn survivor_count(&self) -> usize {
        self.population.len() - self.offspring_count()
    }

    pub fn offspring_count(&self) -> usize {
        (self.population.len() as f32 * self.offspring_fraction) as usize
    }
}
