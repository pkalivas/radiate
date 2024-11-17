use crate::engines::genome::genes::gene::Gene;
use crate::engines::genome::population::Population;
use crate::engines::schema::timer::Timer;

use super::score::Score;
use super::MetricSet;

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
    pub fn score(&self) -> &Score {
        self.score.as_ref().unwrap()
    }

    pub fn seconds(&self) -> f64 {
        self.timer.duration().as_secs_f64()
    }
}

impl<G, A, T> Clone for EngineContext<G, A, T>
where
    G: Gene<G, A>,
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
