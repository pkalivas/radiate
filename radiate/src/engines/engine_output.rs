use crate::engines::genome::population::Population;
use crate::engines::domain::timer::Timer;
use crate::{Chromosome, Metric};
use std::time::Duration;

use super::score::Score;
use super::MetricSet;

/// The context of the genetic engine. This struct contains the current state of the genetic engine
/// at any given time. This includes:
/// * current population
/// * current best individual
/// * current index - the number of generations that have passed
/// * timer - the duration of time the engine has been running
/// * metrics - a set of metrics that are collected during the run
/// * current best score - the score of the current best individual
///
/// The EngineContext is passed to the user-defined closure that is executed each generation. The user
/// can use the EngineContext to access the current state of the genetic engine and make decisions based
/// on the current state on how to proceed.
///
/// # Type Parameters
/// - `C`: The type of chromosome used in the genotype, which must implement the `Chromosome` trait.
/// - `T`: The type of the best individual in the population.
///
pub struct EngineOutput<C, T>
where
    C: Chromosome,
{
    pub population: Population<C>,
    pub best: T,
    pub index: i32,
    pub timer: Timer,
    pub metrics: MetricSet,
    pub score: Option<Score>,
}

impl<C, T> EngineOutput<C, T>
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

    /// Upsert (update or create) a metric with the given key and value. This is only used within the engine itself.
    pub fn upsert_metric(&mut self, metric: Metric) {
        self.metrics.upsert(metric);
    }
}

impl<C, T> Clone for EngineOutput<C, T>
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
        }
    }
}

impl<C, T: std::fmt::Debug> std::fmt::Debug for EngineOutput<C, T>
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
