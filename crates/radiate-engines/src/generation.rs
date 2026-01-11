use crate::Chromosome;
use crate::context::Context;
use radiate_core::objectives::Scored;
use radiate_core::{Ecosystem, Front, MetricSet, Objective, Phenotype, Population, Score, Species};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::time::Duration;

/// A [Generation] represents a single generation in the evolutionary process.
/// It contains the ecosystem, best solution, index, metrics, score, objective,
/// and optionally the Pareto front for multi-objective problems.
///
/// The [Generation] struct is designed to be efficient in terms of memory usage
/// by utilizing reference counting for the ecosystem data when possible. This allows for
/// multiple generations to share the same ecosystem data without unnecessary duplication. However,
/// because of this, the generation's ecosystem is treated as 'copy on read' if it is shared. So,
/// the first time you access the ecosystem, it will be cloned if it is shared.
///
/// This is the main structure returned by the engine after each epoch, and it provides
/// access to all relevant information about that generation.
///
/// # Example
/// ```rust
/// use radiate_core::*;
/// use radiate_engines::*;
/// use std::time::Duration;
///
/// let engine = GeneticEngine::builder()
///     .codec(FloatChromosome::from((10, 0.0..1.0)))
///     .fitness_fn(|vec: Vec<f32>| -vec.iter().map(|x| x * x).sum::<f32>())
///     .build();
///
/// let generation = engine.iter().take(10).last().unwrap();
///
/// let ecosystem: &Ecosystem<FloatChromosome> = generation.ecosystem();
///
/// let population: &Population<FloatChromosome> = generation.population();
/// assert!(population.len() == 100);
///
/// let solution: &Vec<f32> = generation.value();
/// let index: usize = generation.index();
/// let score: &Score = generation.score();
/// let time: Duration = generation.time();
///
/// assert!(solution.len() == 10);
/// assert!(index == 10);
/// ```
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Generation<C, T>
where
    C: Chromosome,
{
    ecosystem: Ecosystem<C>,
    value: T,
    index: usize,
    metrics: MetricSet,
    score: Score,
    objective: Objective,
    front: Option<Front<Phenotype<C>>>,
}

impl<C, T> Generation<C, T>
where
    C: Chromosome,
{
    pub fn score(&self) -> &Score {
        &self.score
    }

    pub fn front(&self) -> Option<&Front<Phenotype<C>>> {
        self.front.as_ref()
    }

    pub fn value(&self) -> &T {
        &self.value
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn metrics(&self) -> &MetricSet {
        &self.metrics
    }

    pub fn objective(&self) -> &Objective {
        &self.objective
    }

    pub fn ecosystem(&self) -> &Ecosystem<C> {
        &self.ecosystem
    }

    pub fn population(&self) -> &Population<C> {
        &self.ecosystem().population()
    }

    pub fn species(&self) -> Option<&[Species<C>]> {
        self.ecosystem().species().map(|s| s.as_slice())
    }

    pub fn time(&self) -> Duration {
        self.metrics()
            .time()
            .map(|m| m.time_statistic().map(|t| t.sum()))
            .flatten()
            .unwrap_or_default()
    }

    pub fn seconds(&self) -> f64 {
        self.time().as_secs_f64()
    }
}

impl<C: Chromosome, T> Scored for Generation<C, T> {
    fn score(&self) -> Option<&Score> {
        Some(&self.score)
    }
}

impl<C, T> From<&Context<C, T>> for Generation<C, T>
where
    C: Chromosome + Clone,
    T: Clone,
{
    fn from(context: &Context<C, T>) -> Self {
        Generation {
            ecosystem: context.ecosystem.clone(),
            value: context.best.clone(),
            index: context.index,
            metrics: context.metrics.clone(),
            score: context.score.clone().unwrap(),
            objective: context.objective.clone(),
            front: match context.objective {
                Objective::Multi(_) => Some(context.front.read().unwrap().clone()),
                _ => None,
            },
        }
    }
}

impl<C, T> Clone for Generation<C, T>
where
    C: Chromosome + Clone,
    T: Clone,
{
    fn clone(&self) -> Self {
        Generation {
            ecosystem: self.ecosystem.clone(),
            value: self.value.clone(),
            index: self.index,
            metrics: self.metrics.clone(),
            score: self.score.clone(),
            objective: self.objective.clone(),
            front: self.front.as_ref().map(|f| f.clone()),
        }
    }
}

impl<C, T> Debug for Generation<C, T>
where
    C: Chromosome,
    T: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ecosystem = &self.ecosystem;

        write!(f, "Generation {{\n")?;
        write!(f, "  metrics: {:?},\n", self.metrics)?;
        write!(f, "  value: {:?},\n", self.value)?;
        write!(f, "  score: {:?},\n", self.score)?;
        write!(f, "  index: {:?},\n", self.index)?;
        write!(f, "  size: {:?},\n", ecosystem.population().len())?;
        write!(f, "  duration: {:?},\n", self.time())?;
        write!(f, "  objective: {:?},\n", self.objective)?;

        if let Some(species) = &ecosystem.species {
            for s in species {
                write!(f, "  species: {:?},\n", s)?;
            }
        }

        write!(f, "}}")
    }
}

impl<C, T> FromIterator<Generation<C, T>> for Front<Phenotype<C>>
where
    C: Chromosome + Clone,
{
    fn from_iter<I: IntoIterator<Item = Generation<C, T>>>(iter: I) -> Self {
        iter.into_iter()
            .last()
            .map(|generation| generation.front().map(|front| front.clone()))
            .flatten()
            .unwrap_or_default()
    }
}
