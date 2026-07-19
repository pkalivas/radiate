use crate::Chromosome;
use crate::context::EvolutionContext;
use radiate_core::objectives::Scored;
use radiate_core::rate::ExprSet;
use radiate_core::{Ecosystem, Front, MetricSet, Objective, Phenotype, Population, Score, Species};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;

/// A [Generation] represents a single generation in the evolutionary process.
/// It contains the ecosystem, the best solution, index, metrics, score, objective,
/// and optionally the Pareto front for multi-objective problems.
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
///     .codec(FloatChromosome::from((10, 0.0_f32..1.0_f32)))
///     .fitness_fn(|vec: Vec<f32>| -vec.iter().map(|x| x * x).sum::<f32>())
///     .build();
///
/// let generation = engine.iter().take(10).last().unwrap();
///
/// let ecosystem: &Ecosystem<FloatChromosome<f32>> = generation.ecosystem();
///
/// let population: &Population<FloatChromosome<f32>> = generation.population();
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
    ecosystem: Arc<Ecosystem<C>>,
    value: T,
    index: usize,
    metrics: Arc<MetricSet>,
    score: Score,
    objective: Objective,
    front: Option<Arc<Front<Phenotype<C>>>>,
    exprs: Option<Arc<Mutex<ExprSet>>>,
}

impl<C, T> Generation<C, T>
where
    C: Chromosome,
{
    pub fn score(&self) -> &Score {
        &self.score
    }

    pub fn front(&self) -> Option<&Front<Phenotype<C>>> {
        self.front.as_deref()
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
        self.ecosystem().population()
    }

    pub fn species(&self) -> Option<&[Species<C>]> {
        self.ecosystem().species().map(|s| s.as_slice())
    }

    pub fn time(&self) -> Duration {
        self.metrics()
            .time()
            .and_then(|m| m.times().map(|t| t.sum()))
            .unwrap_or_default()
    }

    pub fn seconds(&self) -> f64 {
        self.time().as_secs_f64()
    }

    pub fn exprs(&self) -> Option<Arc<Mutex<ExprSet>>> {
        self.exprs.clone()
    }

    pub fn arc_ecosystem(&self) -> Arc<Ecosystem<C>> {
        Arc::clone(&self.ecosystem)
    }

    pub fn arc_metrics(&self) -> Arc<MetricSet> {
        Arc::clone(&self.metrics)
    }
}

impl<C: Chromosome, T> Scored for Generation<C, T> {
    fn score(&self) -> Option<&Score> {
        Some(&self.score)
    }
}

impl<C, T> From<&EvolutionContext<C, T>> for Generation<C, T>
where
    C: Chromosome + Clone,
    T: Clone,
{
    fn from(context: &EvolutionContext<C, T>) -> Self {
        Generation {
            ecosystem: Arc::new(context.ecosystem.clone()),
            value: context.best.clone(),
            index: context.index,
            metrics: Arc::new(context.metrics.clone()),
            score: context.score.clone().unwrap(),
            objective: context.objective.clone(),
            front: match context.objective {
                Objective::Multi(_) => Some(Arc::new(context.front.read().unwrap().clone())),
                _ => None,
            },
            exprs: context.exprs.clone(),
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
            ecosystem: Arc::clone(&self.ecosystem),
            value: self.value.clone(),
            index: self.index,
            metrics: Arc::clone(&self.metrics),
            score: self.score.clone(),
            objective: self.objective.clone(),
            front: self.front.clone(),
            exprs: self.exprs.clone(),
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

        writeln!(f, "Generation {{")?;
        writeln!(f, "  metrics: {:?},", self.metrics)?;
        writeln!(f, "  score: {:?},", self.score)?;
        writeln!(f, "  index: {:?},", self.index)?;
        writeln!(f, "  size: {:?},", ecosystem.population().len())?;
        writeln!(f, "  duration: {:?},", self.time())?;
        writeln!(f, "  objective: {:?},", self.objective)?;
        if let Some(species) = &ecosystem.species {
            writeln!(f, "  species [")?;
            for s in species {
                writeln!(
                    f,
                    "  \t{:?}  age={}",
                    s,
                    self.index.saturating_sub(s.generation()),
                )?;
            }
            writeln!(f, "  ],")?;
        }
        writeln!(f, "  value: {:?},", self.value)?;

        write!(f, "}}")
    }
}

impl<C, T> Default for Generation<C, T>
where
    C: Chromosome + Default,
    T: Default,
{
    fn default() -> Self {
        Generation {
            ecosystem: Arc::new(Ecosystem::default()),
            value: T::default(),
            index: 0,
            metrics: Arc::new(MetricSet::default()),
            score: Score::default(),
            objective: Objective::default(),
            front: None,
            exprs: None,
        }
    }
}

impl<C, T> FromIterator<Generation<C, T>> for Front<Phenotype<C>>
where
    C: Chromosome + Clone,
{
    fn from_iter<I: IntoIterator<Item = Generation<C, T>>>(iter: I) -> Self {
        iter.into_iter()
            .last()
            .and_then(|generation| generation.front().cloned())
            .unwrap_or_default()
    }
}

pub struct GenerationView<'a, C, T>
where
    C: Chromosome,
{
    context: &'a EvolutionContext<C, T>,
}

impl<'a, C, T> GenerationView<'a, C, T>
where
    C: Chromosome,
{
    pub fn new(context: &'a EvolutionContext<C, T>) -> Self {
        Self { context }
    }

    pub fn score(&self) -> &Score {
        self.context.score.as_ref().unwrap()
    }

    pub fn front(&self) -> Arc<RwLock<Front<Phenotype<C>>>> {
        Arc::clone(&self.context.front)
    }

    pub fn value(&self) -> &T {
        &self.context.best
    }

    pub fn phenotype(&self) -> &Phenotype<C> {
        &self.context.ecosystem().population()[0]
    }

    pub fn index(&self) -> usize {
        self.context.index
    }

    pub fn metrics(&self) -> &MetricSet {
        &self.context.metrics
    }

    pub fn objective(&self) -> &Objective {
        &self.context.objective
    }

    pub fn ecosystem(&self) -> &Ecosystem<C> {
        &self.context.ecosystem
    }

    pub fn population(&self) -> &Population<C> {
        self.ecosystem().population()
    }

    pub fn species(&self) -> Option<&[Species<C>]> {
        self.ecosystem().species().map(|s| s.as_slice())
    }

    pub fn time(&self) -> Duration {
        self.metrics()
            .time()
            .and_then(|m| m.times().map(|t| t.sum()))
            .unwrap_or_default()
    }

    pub fn seconds(&self) -> f64 {
        self.time().as_secs_f64()
    }
}
