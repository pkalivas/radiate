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

pub struct AdaptiveParam {
    rate: Box<dyn ParamRate>,
}

impl AdaptiveParam {
    pub fn new(rate: Box<dyn ParamRate>) -> Self {
        AdaptiveParam { rate }
    }

    pub fn get(&self) -> f32 {
        self.rate.get()
    }

    pub fn update(&self, measurement: f32) {
        self.rate.update(measurement);
    }
}

pub trait ParamRate {
    fn get(&self) -> f32;
    fn update(&self, measurement: f32);
}

impl ParamRate for f32 {
    fn get(&self) -> f32 {
        *self
    }

    fn update(&self, _: f32) {}
}

impl Into<AdaptiveParam> for f32 {
    fn into(self) -> AdaptiveParam {
        AdaptiveParam::new(Box::new(self))
    }
}

// use std::cell::Cell;
// use std::sync::Arc;

// /// Adaptive parameters struct for speciation threshold and operator rates.
// pub struct AdaptiveParameters {
//     /// The speciation threshold is adjusted based on diversity in the population.
//     pub threshold: Cell<f32>,
//     /// Mutation rate as a percentage (0.0 - 1.0)
//     pub mutation_rate: Cell<f32>,
//     /// Crossover rate as a percentage (0.0 - 1.0)
//     pub crossover_rate: Cell<f32>,
// }

// impl AdaptiveParameters {
//     /// Update the speciation threshold based on the current distance distribution.
//     /// Here we use a weighted average of the first quartile and the median as an example.
//     pub fn update_threshold(&self, distribution: &crate::Distribution) {
//         // For instance, candidate threshold is average of first quartile and median.
//         let candidate = 0.5 * distribution.first_quartile() + 0.5 * distribution.median();
//         let current = self.threshold.get();
//         let alpha = 0.20; // smoothing factor
//         let updated = alpha * candidate + (1.0 - alpha) * current;
//         self.threshold.set(updated);
//         println!(
//             "Adaptive threshold updated: candidate={:.4}, updated={:.4}",
//             candidate, updated
//         );
//     }

//     /// Update mutation and crossover rates based on population progress.
//     /// For example, if there has been little improvement over recent generations,
//     /// increase mutation rate to encourage exploration.
//     pub fn update_operator_rates(&self, improvement: f32) {
//         // improvement is a measure of how much the best fitness has improved.
//         // For example, if improvement is small, increase mutation.
//         let current_mut = self.mutation_rate.get();
//         let current_cross = self.crossover_rate.get();

//         // If improvement is below some threshold, bump up mutation and lower crossover.
//         if improvement < 0.001 {
//             let new_mut = (current_mut + 0.05).min(1.0);
//             let new_cross = (current_cross - 0.05).max(0.0);
//             self.mutation_rate.set(new_mut);
//             self.crossover_rate.set(new_cross);
//             println!(
//                 "Low improvement detected: mutation_rate increased to {:.2}, crossover_rate decreased to {:.2}",
//                 new_mut, new_cross
//             );
//         } else {
//             // Otherwise, gradually revert to default rates.
//             let default_mut = 0.05;
//             let default_cross = 0.8;
//             let new_mut = current_mut - 0.01;
//             let new_cross = current_cross + 0.01;
//             self.mutation_rate.set(new_mut.max(default_mut));
//             self.crossover_rate.set(new_cross.min(default_cross));
//             println!(
//                 "Good improvement: mutation_rate set to {:.2}, crossover_rate set to {:.2}",
//                 new_mut, new_cross
//             );
//         }
//     }
// }
