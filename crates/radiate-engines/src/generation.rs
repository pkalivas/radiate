use crate::Chromosome;
use crate::context::Context;
use radiate_core::objectives::Scored;
use radiate_core::{
    Ecosystem, Front, MetricSet, Objective, Phenotype, Population, Score, Species, metric_names,
};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::time::Duration;

#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum EcosystemSnapshot<C: Chromosome> {
    Owned(Ecosystem<C>),
    Shared(Ecosystem<C>),
}

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

    pub fn ecosystem(&mut self) -> &Ecosystem<C>
    where
        C: Clone,
    {
        if let EcosystemSnapshot::Owned(ref eco) = self.ecosystem {
            return eco;
        } else if let EcosystemSnapshot::Shared(eco) = &mut self.ecosystem {
            self.ecosystem = EcosystemSnapshot::Owned(eco.clone());
        }

        self.ecosystem()
    }

    pub fn population(&mut self) -> &Population<C>
    where
        C: Clone,
    {
        &self.ecosystem().population()
    }

    pub fn species(&mut self) -> Option<&[Species<C>]>
    where
        C: Clone,
    {
        self.ecosystem().species().map(|s| s.as_slice())
    }

    pub fn time(&self) -> Duration {
        self.metrics()
            .get(metric_names::TIME)
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

impl<C: Chromosome + Clone, T: Clone> From<&Context<C, T>> for Generation<C, T> {
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

impl<C: Chromosome, T: Debug> Debug for Generation<C, T> {
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
