use crate::Chromosome;
use crate::context::Context;
use radiate_core::objectives::Scored;
use radiate_core::{Ecosystem, Front, MetricSet, Objective, Phenotype, Population, Score, Species};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::time::Duration;

/// A snapshot of an ecosystem, either owned or shared.
///
/// Owned ecosystems contain their own data, while shared ecosystems
/// contain reference counted clones of the data. This allows for
/// efficient sharing of ecosystems between generations without
/// unnecessary cloning. However, this means that a shared ecosystem
/// should not be modified directly, as it may affect other generations
/// that share the same data.
#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum EcosystemSnapshot<C: Chromosome> {
    Owned(Ecosystem<C>),
    Shared(Ecosystem<C>),
}

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
/// let mut generation = engine.iter().take(10).last().unwrap();
///
/// {
///     // triggers a clone of the ecosystem if it is shared. it is in this case.
///     let ecosystem: &Ecosystem<FloatChromosome> = generation.ecosystem();
/// }
///
/// {
///     // Would trigger a clone of the ecosystem if it is shared. It is NOT in this case
///     // because it was just converted to an owned ecosystem above.
///     let population: &Population<FloatChromosome> = generation.population();
///     assert!(population.len() == 100);
/// }
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
    ecosystem: EcosystemSnapshot<C>,
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

    /// Access the ecosystem, cloning it if it is shared. When this is called,
    /// if the ecosystem is in the [EcosystemSnapshot::Shared] variant, it
    /// will be cloned into the [EcosystemSnapshot::Owned] variant for future
    /// accesses. When the generation is created from a [Context], the ecosystem
    /// is always in the shared variant to avoid unnecessary cloning of the ecosystem.
    pub fn ecosystem(&mut self) -> &Ecosystem<C>
    where
        C: Clone,
    {
        if let EcosystemSnapshot::Owned(ref eco) = self.ecosystem {
            return eco;
        } else if let EcosystemSnapshot::Shared(eco) = &self.ecosystem {
            self.ecosystem = EcosystemSnapshot::Owned(eco.clone());
        }

        self.ecosystem()
    }

    /// Access the population from the ecosystem. Just like [Generation::ecosystem],
    /// if the ecosystem is shared, it will be cloned on first access.
    pub fn population(&mut self) -> &Population<C>
    where
        C: Clone,
    {
        &self.ecosystem().population()
    }

    /// Access the species from the ecosystem. Just like [Generation::ecosystem],
    /// if the ecosystem is shared, it will be cloned on first access.
    pub fn species(&mut self) -> Option<&[Species<C>]>
    where
        C: Clone,
    {
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
            ecosystem: EcosystemSnapshot::Shared(Ecosystem::clone_ref(&context.ecosystem)),
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
            ecosystem: match &self.ecosystem {
                EcosystemSnapshot::Owned(eco) => EcosystemSnapshot::Owned(eco.clone()),
                EcosystemSnapshot::Shared(eco) => EcosystemSnapshot::Owned(eco.clone()),
            },
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
        let ecosystem = match &self.ecosystem {
            EcosystemSnapshot::Owned(eco) => eco,
            EcosystemSnapshot::Shared(eco) => eco,
        };

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
