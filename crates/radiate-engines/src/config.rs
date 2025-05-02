use radiate_core::{Diversity, Genotype};

use super::thread_pool::ThreadPool;
use super::{Alter, Audit, Front, Problem, ReplacementStrategy, Select};
use crate::Chromosome;
use crate::genome::phenotype::Phenotype;
use crate::genome::population::Population;
use crate::objectives::Objective;
use crate::steps::Evaluator;
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct EngineConfig<C: Chromosome, T> {
    pub(crate) population: Population<C>,
    pub(crate) problem: Arc<dyn Problem<C, T>>,
    pub(crate) survivor_selector: Arc<dyn Select<C>>,
    pub(crate) offspring_selector: Arc<dyn Select<C>>,
    pub(crate) replacement_strategy: Arc<dyn ReplacementStrategy<C>>,
    pub(crate) audits: Vec<Arc<dyn Audit<C>>>,
    pub(crate) alterers: Vec<Arc<dyn Alter<C>>>,
    pub(crate) species_threshold: f32,
    pub(crate) diversity: Option<Arc<dyn Diversity<C>>>,
    pub(crate) evaluator: Arc<dyn Evaluator<C, T>>,
    pub(crate) objective: Objective,
    pub(crate) thread_pool: Arc<ThreadPool>,
    pub(crate) max_age: usize,
    pub(crate) max_species_age: usize,
    pub(crate) front: Arc<RwLock<Front<Phenotype<C>>>>,
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

    pub fn encoder(&self) -> Arc<dyn Fn() -> Genotype<C> + Send + Sync>
    where
        C: 'static,
        T: 'static,
    {
        let problem = Arc::clone(&self.problem);
        Arc::new(move || problem.encode())
    }

    pub fn evaluator(&self) -> Arc<dyn Evaluator<C, T>>
    where
        C: 'static,
        T: Send + Sync + 'static,
    {
        Arc::clone(&self.evaluator)
    }
}
