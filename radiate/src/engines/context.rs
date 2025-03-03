use super::objectives::Score;
use super::{EngineError, MetricSet};
use crate::Chromosome;
use crate::engines::domain::timer::Timer;
use crate::engines::genome::population::Population;
use crate::objectives::Front;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};
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
    pub population: Population<C>,
    pub best: T,
    pub index: usize,
    pub timer: Timer,
    pub metrics: MetricSet,
    pub score: Option<Score>,
    pub front: Arc<Mutex<Front>>,
    pub error: Option<EngineError>,
}

impl<C, T> EngineContext<C, T>
where
    C: Chromosome,
{
    /// Get the current score of the best individual in the population.
    pub fn score(&self) -> Score {
        match self.error {
            Some(_) => Score::default(),
            None => self.score.clone().unwrap_or_default(),
        }
    }

    /// Get the current duration of the genetic engine run in seconds.
    pub fn seconds(&self) -> f64 {
        self.timer.duration().as_secs_f64()
    }

    /// Upsert (update or create) a metric operation with the given name, value, and time.
    pub fn upsert_operation(
        &mut self,
        name: &'static str,
        value: impl Into<f32>,
        time: impl Into<Duration>,
    ) {
        self.metrics.upsert_operations(name, value, time);
    }

    pub fn is_ok(&self) -> bool {
        self.error.is_none()
    }

    pub fn is_err(&self) -> bool {
        self.error.is_some()
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
            error: self.error.clone(),
        }
    }
}

impl<C, T: Debug> Debug for EngineContext<C, T>
where
    C: Chromosome,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.error {
            Some(ref err) => write!(f, "EngineContext {{\n  error: {:?},\n}}", err),
            None => {
                write!(f, "EngineContext {{\n")?;
                write!(f, "  best: {:?},\n", self.best)?;
                write!(f, "  score: {:?},\n", self.score())?;
                write!(f, "  index: {:?},\n", self.index)?;
                write!(f, "  size: {:?},\n", self.population.len())?;
                write!(f, "  duration: {:?},\n", self.timer.duration())?;
                write!(f, "  metrics: {:?},\n", self.metrics)?;
                write!(f, "}}")
            }
        }
    }
}

pub struct PopulationContext<C>
where
    C: Chromosome,
{
    pub population: Option<Population<C>>,
    pub error: Option<String>,
}

impl<C> PopulationContext<C>
where
    C: Chromosome,
{
    pub fn new(population: Population<C>) -> Self {
        Self {
            population: Some(population),
            error: None,
        }
    }

    pub fn new_error(error: String) -> Self {
        Self {
            population: None,
            error: Some(error),
        }
    }

    pub fn population(&self) -> Option<&Population<C>> {
        self.population.as_ref()
    }

    pub fn error(&self) -> Option<&String> {
        self.error.as_ref()
    }

    pub fn take_population(self) -> Option<Population<C>> {
        self.population
    }

    pub fn take_error(&mut self) -> Option<String> {
        self.error.take()
    }

    pub fn is_ok(&self) -> bool {
        self.error.is_none()
    }
}
