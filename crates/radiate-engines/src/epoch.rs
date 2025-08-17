use crate::Chromosome;
use radiate_core::objectives::Scored;
use radiate_core::{
    Ecosystem, Front, MetricSet, Objective, Phenotype, Population, Problem, Score, Species,
    metric_names,
};
use std::fmt::Debug;
use std::sync::{Arc, RwLock};
use std::time::Duration;

pub struct Context<C: Chromosome, T> {
    pub(crate) ecosystem: Ecosystem<C>,
    pub(crate) best: T,
    pub(crate) index: usize,
    pub(crate) metrics: MetricSet,
    pub(crate) epoch_metrics: MetricSet,
    pub(crate) score: Option<Score>,
    pub(crate) front: Arc<RwLock<Front<Phenotype<C>>>>,
    pub(crate) objective: Objective,
    pub(crate) problem: Arc<dyn Problem<C, T>>,
}

impl<C, T> Clone for Context<C, T>
where
    C: Chromosome + Clone,
    T: Clone,
{
    fn clone(&self) -> Self {
        Context {
            ecosystem: self.ecosystem.clone(),
            best: self.best.clone(),
            index: self.index,
            metrics: self.metrics.clone(),
            epoch_metrics: self.epoch_metrics.clone(),
            score: self.score.clone(),
            front: self.front.clone(),
            objective: self.objective.clone(),
            problem: Arc::clone(&self.problem),
        }
    }
}

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

impl<C: Chromosome, T> Generation<C, T> {
    pub fn score(&self) -> &Score {
        &self.score
    }

    pub fn front(&self) -> Option<&Front<Phenotype<C>>> {
        self.front.as_ref()
    }

    pub fn ecosystem(&self) -> &Ecosystem<C> {
        &self.ecosystem
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

    pub fn population(&self) -> &Population<C> {
        &self.ecosystem().population()
    }

    pub fn species(&self) -> Option<&[Species<C>]> {
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

impl<C: Chromosome, T: Debug> Debug for Generation<C, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Generation {{\n")?;
        write!(f, "  value: {:?},\n", self.value)?;
        write!(f, "  score: {:?},\n", self.score)?;
        write!(f, "  index: {:?},\n", self.index)?;
        write!(f, "  size: {:?},\n", self.ecosystem.population.len())?;
        write!(f, "  duration: {:?},\n", self.time())?;

        if let Some(species) = &self.ecosystem.species {
            for s in species {
                write!(f, "  species: {:?},\n", s)?;
            }
        }

        write!(f, "  metrics: {:?},\n", self.metrics)?;
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
