use radiate_core::Genotype;

use super::thread_pool::ThreadPool;
use super::{Alter, Audit, Front, Problem, ReplacementStrategy, Select};
use crate::Chromosome;
use crate::genome::phenotype::Phenotype;
use crate::genome::population::Population;
use crate::objectives::Objective;
use std::sync::Arc;

pub struct EngineConfig<C: Chromosome, T> {
    pub(crate) population: Population<C>,
    pub(crate) problem: Arc<dyn Problem<C, T>>,
    pub(crate) survivor_selector: Arc<dyn Select<C>>,
    pub(crate) offspring_selector: Arc<dyn Select<C>>,
    pub(crate) replacement_strategy: Arc<dyn ReplacementStrategy<C>>,
    pub(crate) audits: Vec<Arc<dyn Audit<C>>>,
    pub(crate) alterers: Vec<Arc<dyn Alter<C>>>,
    pub(crate) objective: Objective,
    pub(crate) thread_pool: Arc<ThreadPool>,
    pub(crate) max_age: usize,
    pub(crate) front: Front<Phenotype<C>>,
    pub(crate) offspring_fraction: f32,
}

impl<C: Chromosome, T> EngineConfig<C, T> {
    pub fn population(&self) -> &Population<C> {
        &self.population
    }

    pub fn problem(&self) -> Arc<dyn Problem<C, T>> {
        Arc::clone(&self.problem)
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

    pub fn audits(&self) -> &[Arc<dyn Audit<C>>] {
        &self.audits
    }

    pub fn alters(&self) -> &[Arc<dyn Alter<C>>] {
        &self.alterers
    }

    pub fn objective(&self) -> Objective {
        self.objective.clone()
    }

    pub fn thread_pool(&self) -> Arc<ThreadPool> {
        Arc::clone(&self.thread_pool)
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

    pub fn encoder(&self) -> Arc<dyn Fn() -> Genotype<C>>
    where
        C: 'static,
        T: 'static,
    {
        let problem = Arc::clone(&self.problem);
        Arc::new(move || problem.encode())
    }
}

#[derive(Clone)]
pub struct GeneticEngineParams<C: Chromosome, T> {
    population: Population<C>,
    problem: Arc<dyn Problem<C, T>>,
    survivor_selector: Arc<dyn Select<C>>,
    offspring_selector: Arc<dyn Select<C>>,
    replacement_strategy: Arc<dyn ReplacementStrategy<C>>,
    audits: Vec<Arc<dyn Audit<C>>>,
    alterers: Vec<Arc<dyn Alter<C>>>,
    objective: Objective,
    thread_pool: Arc<ThreadPool>,
    max_age: usize,
    front: Front<Phenotype<C>>,
    offspring_fraction: f32,
}

impl<C: Chromosome, T> GeneticEngineParams<C, T> {
    pub fn new(
        population: Population<C>,
        problem: Arc<dyn Problem<C, T>>,
        survivor_selector: Arc<dyn Select<C>>,
        offspring_selector: Arc<dyn Select<C>>,
        replacement_strategy: Arc<dyn ReplacementStrategy<C>>,
        audits: Vec<Arc<dyn Audit<C>>>,
        alterers: Vec<Arc<dyn Alter<C>>>,
        objective: Objective,
        thread_pool: Arc<ThreadPool>,
        max_age: usize,
        front: Front<Phenotype<C>>,
        offspring_fraction: f32,
    ) -> Self {
        GeneticEngineParams {
            population,
            problem,
            survivor_selector,
            offspring_selector,
            replacement_strategy,
            audits,
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

    pub fn alters(&self) -> &[Arc<dyn Alter<C>>] {
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
