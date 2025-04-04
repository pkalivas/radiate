use crate::domain::timer::Timer;
use crate::genome::population::Population;
use crate::objectives::Front;
use crate::sync::{RwCell, RwCellGuard};
use crate::{Chromosome, Genotype, MetricSet, Objective, Phenotype, Score, Scored, Species};
use std::fmt::Debug;
use std::sync::Arc;
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
    pub(crate) front: RwCell<Front<Phenotype<C>>>,
    pub(crate) species: Vec<Species<C>>,
    pub(crate) objective: Objective,
    pub(crate) decoder: Arc<dyn Fn(&Genotype<C>) -> T>,
}

impl<C, T> EngineContext<C, T>
where
    C: Chromosome,
{
    pub fn score(&self) -> &Score {
        self.score.as_ref().unwrap()
    }

    pub fn seconds(&self) -> f64 {
        self.timer.duration().as_secs_f64()
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn population(&self) -> &Population<C> {
        &self.population
    }

    pub fn population_mut(&mut self) -> &mut Population<C> {
        &mut self.population
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

    pub fn pareto_front(&self) -> RwCellGuard<Front<Phenotype<C>>> {
        self.front.read()
    }

    pub fn objective(&self) -> &Objective {
        &self.objective
    }

    pub fn species(&self) -> &[Species<C>] {
        &self.species
    }

    pub(crate) fn begin_epoch(&mut self) {
        self.timer.start();
    }

    pub(crate) fn complete_epoch(&mut self) {
        let current_best = self.population.get(0);

        if let (Some(best), Some(current)) = (current_best.score().as_ref(), &self.score) {
            if self.objective().is_better(best, &current) {
                self.score = Some((*best).clone());
                self.best = (self.decoder)(&current_best.genotype());
            }
        } else {
            self.score = Some(current_best.score().unwrap().clone());
            self.best = (self.decoder)(&current_best.genotype());
        }

        self.index += 1;
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
            front: RwCell::clone(&self.front),
            species: self.species.clone(),
            objective: self.objective.clone(),
            decoder: Arc::clone(&self.decoder),
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
