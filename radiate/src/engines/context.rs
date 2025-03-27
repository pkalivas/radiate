use super::objectives::Score;
use super::{Metric, MetricSet, Phenotype, Species};
use crate::Chromosome;
use crate::engines::domain::timer::Timer;
use crate::engines::genome::population::Population;
use crate::objectives::Front;
use std::fmt::Debug;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

pub struct EngineContext<C, T>
where
    C: Chromosome,
    T: Clone,
{
    pub population: Population<C>,
    pub best: T,
    pub index: usize,
    pub timer: Timer,
    pub metrics: MetricSet,
    pub score: Option<Score>,
    pub front: Arc<Mutex<Front<Phenotype<C>>>>,
    pub species: Vec<Species<C>>,
}

impl<C, T> EngineContext<C, T>
where
    C: Chromosome,
    T: Clone,
{
    pub fn score(&self) -> Score {
        self.score.clone().unwrap()
    }

    pub fn seconds(&self) -> f64 {
        self.timer.duration().as_secs_f64()
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn timer(&self) -> Timer {
        self.timer.clone()
    }

    pub fn front(&self) -> MutexGuard<'_, Front<Phenotype<C>>> {
        self.front.lock().expect("Failed to lock front")
    }
}

impl<C, T> Clone for EngineContext<C, T>
where
    C: Chromosome,
    T: Clone,
{
    fn clone(&self) -> Self {
        EngineContext {
            population: self.population.clone(),
            best: self.best.clone(),
            index: self.index,
            timer: self.timer.clone(),
            metrics: self.metrics.clone(),
            score: self.score.clone(),
            front: self.front.clone(),
            species: self.species.clone(),
        }
    }
}

impl<C, T> Debug for EngineContext<C, T>
where
    C: Chromosome,
    T: Debug + Clone,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EngineOutput {{\n")?;
        write!(f, "  best: {:?},\n", self.best)?;
        write!(f, "  score: {:?},\n", self.score)?;
        write!(f, "  index: {:?},\n", self.index)?;
        write!(f, "  size: {:?},\n", self.population.len())?;
        write!(f, "  duration: {:?},\n", self.timer.duration())?;
        write!(f, "  metrics: {:?},\n", self.metrics)?;

        let species = &self.species;
        if !species.is_empty() {
            write!(f, "  species: [\n")?;
            for s in species.iter() {
                write!(f, "    {:?},\n", s)?;
            }
            write!(f, "  ],\n")?;
        }

        write!(f, "}}")
    }
}

impl<C, T> From<SharedEngineContext<C, T>> for EngineContext<C, T>
where
    C: Chromosome,
    T: Clone,
{
    fn from(ctx: SharedEngineContext<C, T>) -> Self {
        let population = ctx.population_lock().take(|_| true);
        let best = match Arc::try_unwrap(ctx.best) {
            Ok(best) => best.into_inner().unwrap(),
            Err(best) => best.lock().unwrap().clone(),
        };
        let metrics = match Arc::try_unwrap(ctx.metrics) {
            Ok(metrics) => metrics.into_inner().unwrap(),
            Err(metrics) => metrics.lock().unwrap().clone(),
        };
        let species = match Arc::try_unwrap(ctx.species) {
            Ok(species) => species.into_inner().unwrap(),
            Err(species) => species.lock().unwrap().clone(),
        };
        // let front = match Arc::try_unwrap(ctx.front) {
        //     Ok(front) => front.into_inner().unwrap(),
        //     Err(front) => front.lock().unwrap().clone(),
        // };

        EngineContext {
            population,
            best,
            index: *ctx.index.lock().unwrap(),
            timer: ctx.timer.lock().unwrap().clone(),
            metrics,
            score: ctx.score.lock().unwrap().clone(),
            front: Arc::clone(&ctx.front),
            species,
        }
    }
}

/// The context of the genetic engine. This struct contains the current state of the genetic engine
/// at any given time. This includes:
/// * current population
/// * current best individual
/// * current index - the number of generations that have passed
/// * timer - the duration of time the engine has been running
/// * metrics - a set of metrics that are collected during the run
/// * current best score - the score of the current best individual
/// * front - the current pareto front of the population (if multi-objective)
///
/// The EngineContext is passed to the user-defined closure that is executed each generation. The user
/// can use the EngineContext to access the current state of the genetic engine and make decisions based
/// on the current state on how to proceed.
///
/// # Type Parameters
/// - `C`: The type of chromosome used in the genotype, which must implement the `Chromosome` trait.
/// - `T`: The type of the best individual in the population.
pub struct SharedEngineContext<C, T>
where
    C: Chromosome,
{
    pub population: Arc<Mutex<Population<C>>>,
    pub best: Arc<Mutex<T>>,
    pub index: Arc<Mutex<usize>>,
    pub timer: Arc<Mutex<Timer>>,
    pub metrics: Arc<Mutex<MetricSet>>,
    pub score: Arc<Mutex<Option<Score>>>,
    pub front: Arc<Mutex<Front<Phenotype<C>>>>,
    pub species: Arc<Mutex<Vec<Species<C>>>>,
}

impl<C, T> SharedEngineContext<C, T>
where
    C: Chromosome,
{
    pub fn score(&self) -> Score {
        self.score.lock().unwrap().clone().unwrap()
    }

    pub fn seconds(&self) -> f64 {
        self.timer.lock().unwrap().duration().as_secs_f64()
    }

    pub fn index(&self) -> usize {
        *self.index.lock().unwrap()
    }

    pub fn timer(&self) -> Timer {
        self.timer.lock().unwrap().clone()
    }

    pub fn metrics_lock(&self) -> MutexGuard<'_, MetricSet> {
        self.metrics.lock().expect("Failed to lock metrics")
    }

    pub fn best_lock(&self) -> MutexGuard<'_, T> {
        self.best.lock().expect("Failed to lock best")
    }

    pub fn upsert_operation(
        &self,
        name: &'static str,
        value: impl Into<f32>,
        time: impl Into<Duration>,
    ) {
        self.metrics_lock().upsert_operations(name, value, time);
    }

    pub fn upsert_distribution(&self, name: &'static str, values: &[f32]) {
        self.metrics_lock().upsert_sequence(name, values);
    }

    pub fn upsert_metric(&self, metric: Metric) {
        self.metrics_lock().upsert(metric);
    }

    pub fn population_lock(&self) -> MutexGuard<'_, Population<C>> {
        self.population.lock().expect("Failed to lock population")
    }

    pub fn population_len(&self) -> usize {
        self.population.lock().unwrap().len()
    }

    pub fn phenotype(&self, index: usize) -> Phenotype<C> {
        self.population.lock().unwrap()[index].clone()
    }

    pub fn set_species_id(&self, index: usize, species_id: u64) {
        self.population.lock().unwrap()[index].set_species_id(Some(species_id));
    }

    pub fn add_species(&self, species: Species<C>) {
        self.species.lock().unwrap().push(species);
    }

    pub fn species_lock(&self) -> MutexGuard<'_, Vec<Species<C>>> {
        self.species.lock().expect("Failed to lock species")
    }

    pub fn front(&self) -> MutexGuard<'_, Front<Phenotype<C>>> {
        self.front.lock().expect("Failed to lock front")
    }

    pub fn update_index(&self) {
        let mut idx = self.index.lock().unwrap();
        *idx += 1;
    }

    pub fn update_score(&self, score: Score) {
        let mut sc = self.score.lock().unwrap();
        *sc = Some(score);
    }

    pub fn update_best(&self, best: T) {
        let mut b = self.best.lock().unwrap();
        *b = best;
    }

    pub fn stop_timer(&self) {
        self.timer.lock().unwrap().stop();
    }
}

impl<C, T> From<EngineContext<C, T>> for SharedEngineContext<C, T>
where
    C: Chromosome,
    T: Clone,
{
    fn from(ctx: EngineContext<C, T>) -> Self {
        let population = Arc::new(Mutex::new(ctx.population));
        let best = Arc::new(Mutex::new(ctx.best));
        let index = Arc::new(Mutex::new(ctx.index));
        let timer = Arc::new(Mutex::new(ctx.timer));
        let metrics = Arc::new(Mutex::new(ctx.metrics));
        let score = Arc::new(Mutex::new(ctx.score));
        let species = Arc::new(Mutex::new(ctx.species));

        SharedEngineContext {
            population,
            best,
            index,
            timer,
            metrics,
            score,
            front: ctx.front,
            species,
        }
    }
}

impl<C, T> Clone for SharedEngineContext<C, T>
where
    C: Chromosome,
    T: Clone,
{
    fn clone(&self) -> Self {
        Self {
            population: self.population.clone(),
            best: self.best.clone(),
            index: self.index.clone(),
            timer: self.timer.clone(),
            metrics: self.metrics.clone(),
            score: self.score.clone(),
            front: self.front.clone(),
            species: self.species.clone(),
        }
    }
}

impl<C, T: Debug> Debug for SharedEngineContext<C, T>
where
    C: Chromosome,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EngineOutput {{\n")?;
        write!(f, "  best: {:?},\n", self.best)?;
        write!(f, "  score: {:?},\n", self.score())?;
        write!(f, "  index: {:?},\n", self.index)?;
        write!(f, "  size: {:?},\n", self.population_len())?;
        write!(
            f,
            "  duration: {:?},\n",
            self.timer.lock().unwrap().duration()
        )?;
        write!(f, "  metrics: {:?},\n", self.metrics)?;

        let species = self.species.lock().unwrap();
        if !species.is_empty() {
            write!(f, "  species: [\n")?;
            for s in species.iter() {
                write!(f, "    {:?},\n", s)?;
            }
            write!(f, "  ],\n")?;
        }

        write!(f, "}}")
    }
}
