use super::MetricSet;
use super::objectives::Score;
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

    /// Upsert (update or create) a metric operation with the given name, value, and time.
    pub fn upsert_operation(&mut self, name: &'static str, value: f32, time: Duration) {
        self.metrics.upsert_operations(name, value, time);
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
        write!(f, "}}")
    }
}
