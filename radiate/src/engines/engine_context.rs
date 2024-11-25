use std::time::Duration;

use crate::engines::genome::genes::gene::Gene;
use crate::engines::genome::population::Population;
use crate::engines::schema::timer::Timer;

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
/// - `G`: The type of gene used in the genetic algorithm, which must implement the `Gene` trait.
/// - `A`: The type of the allele associated with the gene - the gene's "expression".
/// - `T`: The type of the best individual in the population.
///
pub struct EngineContext<G, A, T>
where
    G: Gene<G, A>,
{
    pub population: Population<G, A>,
    pub best: T,
    pub index: i32,
    pub timer: Timer,
    pub metrics: MetricSet,
    pub score: Option<Score>,
}

impl<G, A, T> EngineContext<G, A, T>
where
    G: Gene<G, A>,
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
    pub fn upsert_metric(&mut self, key: &'static str, value: f32, time: Option<Duration>) {
        self.metrics.upsert_value(key, value);
        if let Some(time) = time {
            self.metrics.upsert_time(key, time);
        }
    }
}

impl<G, A, T> Clone for EngineContext<G, A, T>
where
    G: Gene<G, A>,
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

impl<G, A, T: std::fmt::Debug> std::fmt::Debug for EngineContext<G, A, T>
where
    G: Gene<G, A>,
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
