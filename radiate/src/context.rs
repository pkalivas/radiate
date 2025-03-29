use crate::domain::timer::Timer;
use crate::genome::population::Population;
use crate::objectives::Front;
use crate::{Chromosome, Metric, MetricSet, Phenotype, Score, Species};
use std::fmt::Debug;
use std::time::Duration;

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
/// It can be thought of as a snapshot of the genetic engine's state at a specific generation.
///
/// The EngineContext is passed to the user-defined closure that is executed each generation. The user
/// can use the EngineContext to access the current state of the genetic engine and make decisions based
/// on the current state on how to proceed.
///
/// # Type Parameters
/// - `C`: The type of chromosome used in the genotype, which must implement the `Chromosome` trait.
/// - `T`: The type of the best individual in the population.
pub struct EngineContext<C, T>
where
    C: Chromosome,
{
    pub(crate) population: Population<C>,
    pub(crate) best: T,
    pub(crate) index: usize,
    pub(crate) timer: Timer,
    pub(crate) metrics: MetricSet,
    pub(crate) score: Option<Score>,
    pub(crate) front: Front<Phenotype<C>>,
    pub(crate) species: Vec<Species<C>>,
}

/// Encapsulates information about the best solution found so far.
pub struct BestSolution<T> {
    pub individual: T,
    pub score: Score,
}

impl<C, T> EngineContext<C, T>
where
    C: Chromosome,
{
    /// Get the current score of the best individual in the population.
    pub fn score(&self) -> &Score {
        self.score.as_ref().unwrap()
    }

    /// Get the current duration of the genetic engine run in seconds.
    pub fn seconds(&self) -> f64 {
        self.timer.duration().as_secs_f64()
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn population(&self) -> &Population<C> {
        &self.population
    }

    pub fn best(&self) -> &T {
        &self.best
    }

    pub fn duration(&self) -> Duration {
        self.timer.duration()
    }

    pub fn metrics(&self) -> &MetricSet {
        &self.metrics
    }

    pub fn pareto_front(&self) -> &Front<Phenotype<C>> {
        &self.front
    }

    /// Upsert (update or create) a metric operation with the given name, value, and time.
    pub(crate) fn record_operation(
        &mut self,
        name: &'static str,
        value: impl Into<f32>,
        time: impl Into<Duration>,
    ) {
        self.metrics.upsert_operations(name, value, time);
    }

    pub(crate) fn record_distribution(&mut self, name: &'static str, values: &[f32]) {
        self.metrics.upsert_sequence(name, values);
    }

    pub(crate) fn record_metric(&mut self, metric: Metric) {
        self.metrics.upsert(metric);
    }

    pub(crate) fn set_species_id(&mut self, index: usize, species_id: u64) {
        self.population[index].set_species_id(Some(species_id));
    }

    pub(crate) fn get_species(&self, idx: usize) -> &Species<C> {
        &self.species[idx]
    }

    pub(crate) fn add_species(&mut self, species: Species<C>) {
        self.species.push(species);
    }

    pub(crate) fn species(&self) -> &[Species<C>] {
        &self.species
    }
}

impl<C, T> Clone for EngineContext<C, T>
where
    C: Chromosome,
    T: Clone,
{
    fn clone(&self) -> Self {
        Self {
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

impl<C, T: Debug> Debug for EngineContext<C, T>
where
    C: Chromosome,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EngineOutput {{\n")?;
        write!(f, "  best: {:?},\n", self.best)?;
        write!(f, "  score: {:?},\n", self.score())?;
        write!(f, "  index: {:?},\n", self.index)?;
        write!(f, "  size: {:?},\n", self.population.len())?;
        write!(f, "  duration: {:?},\n", self.timer.duration())?;
        write!(f, "  metrics: {:?},\n", self.metrics)?;

        if !self.species.is_empty() {
            write!(f, "  species: [\n")?;
            for species in &self.species {
                write!(f, "    {:?},\n", species)?;
            }
            write!(f, "  ],\n")?;
        }

        write!(f, "}}")
    }
}
