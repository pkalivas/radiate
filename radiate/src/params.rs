use super::thread_pool::ThreadPool;
use super::{Alter, DiversityMeasure, Front, Problem, ReplacementStrategy, Select};
use crate::Chromosome;
use crate::genome::phenotype::Phenotype;
use crate::genome::population::Population;
use crate::objectives::Objective;
use crate::sync::RwCell;
use std::sync::Arc;

pub struct GeneticEngineParams<C: Chromosome, T> {
    population: Population<C>,
    problem: Arc<dyn Problem<C, T>>,
    survivor_selector: Arc<dyn Select<C>>,
    offspring_selector: Arc<dyn Select<C>>,
    replacement_strategy: Arc<dyn ReplacementStrategy<C>>,
    distance: Option<Arc<dyn DiversityMeasure<C>>>,
    alterers: Vec<Arc<dyn Alter<C>>>,
    objective: Objective,
    thread_pool: Arc<ThreadPool>,
    max_age: usize,
    max_species_age: usize,
    front: RwCell<Front<Phenotype<C>>>,
    offspring_fraction: f32,
    species_threshold: f32,
}

impl<C: Chromosome, T> GeneticEngineParams<C, T> {
    pub fn new(
        population: Population<C>,
        problem: Arc<dyn Problem<C, T>>,
        survivor_selector: Arc<dyn Select<C>>,
        offspring_selector: Arc<dyn Select<C>>,
        replacement_strategy: Arc<dyn ReplacementStrategy<C>>,
        distance: Option<Arc<dyn DiversityMeasure<C>>>,
        alterers: Vec<Arc<dyn Alter<C>>>,
        objective: Objective,
        thread_pool: Arc<ThreadPool>,
        max_age: usize,
        max_species_age: usize,
        front: RwCell<Front<Phenotype<C>>>,
        offspring_fraction: f32,
        species_threshold: f32,
    ) -> Self {
        GeneticEngineParams {
            population,
            problem,
            survivor_selector,
            offspring_selector,
            replacement_strategy,
            distance,
            alterers,
            objective,
            thread_pool,
            max_age,
            max_species_age,
            front,
            offspring_fraction,
            species_threshold,
        }
    }

    pub fn population(&self) -> &Population<C> {
        &self.population
    }

    pub fn problem(&self) -> Arc<dyn Problem<C, T>> {
        Arc::clone(&self.problem)
    }

    pub fn survivor_selector(&self) -> &Arc<dyn Select<C>> {
        &self.survivor_selector
    }

    pub fn offspring_selector(&self) -> &Arc<dyn Select<C>> {
        &self.offspring_selector
    }

    pub fn replacement_strategy(&self) -> &Arc<dyn ReplacementStrategy<C>> {
        &self.replacement_strategy
    }

    pub fn distance(&self) -> Option<Arc<dyn DiversityMeasure<C>>> {
        self.distance.clone()
    }

    pub fn alters(&self) -> &[Arc<dyn Alter<C>>] {
        &self.alterers
    }

    pub fn objective(&self) -> &Objective {
        &self.objective
    }

    pub fn thread_pool(&self) -> &Arc<ThreadPool> {
        &self.thread_pool
    }

    pub fn max_age(&self) -> usize {
        self.max_age
    }

    pub fn front(&self) -> &RwCell<Front<Phenotype<C>>> {
        &self.front
    }

    pub fn survivor_count(&self) -> usize {
        self.population.len() - self.offspring_count()
    }

    pub fn offspring_count(&self) -> usize {
        (self.population.len() as f32 * self.offspring_fraction) as usize
    }

    pub fn species_threshold(&self) -> f32 {
        self.species_threshold
    }

    pub fn max_species_age(&self) -> usize {
        self.max_species_age
    }
}
