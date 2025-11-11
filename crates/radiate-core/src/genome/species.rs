use super::{Chromosome, Phenotype, Population};
use crate::{Objective, Score, objectives::Scored, tracker::Tracker};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::{
    fmt::{self, Debug, Formatter},
    sync::atomic::{AtomicU64, Ordering},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[repr(transparent)]
pub struct SpeciesId(pub u64);

impl SpeciesId {
    pub fn new() -> Self {
        static SPECIES_ID: AtomicU64 = AtomicU64::new(0);
        SpeciesId(SPECIES_ID.fetch_add(1, Ordering::Relaxed))
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Species<C: Chromosome> {
    pub id: SpeciesId,
    pub generation: usize,
    pub tracker: Tracker<Score>,
    pub score: Option<Score>,
    pub mascot: Phenotype<C>,
    pub population: Population<C>,
}

impl<C: Chromosome> Species<C> {
    pub fn new(generation: usize, initial: &Phenotype<C>) -> Self
    where
        C: Clone,
    {
        Species {
            id: SpeciesId::new(),
            generation,
            tracker: Tracker::new(),
            score: Some(initial.score().unwrap().clone()),
            mascot: initial.clone(),
            population: Population::new(vec![initial.clone()]),
        }
    }

    pub fn clone_ref(other: &Self) -> Self
    where
        C: Clone,
    {
        Species {
            id: other.id,
            generation: other.generation,
            tracker: other.tracker.clone(),
            score: other.score.clone(),
            mascot: other.mascot.clone(),
            population: other.population.clone(),
        }
    }

    pub fn id(&self) -> SpeciesId {
        self.id
    }

    pub fn push(&mut self, individual: Phenotype<C>) {
        self.population.push(individual);
    }

    pub fn population(&self) -> &Population<C> {
        &self.population
    }

    pub fn stagnation(&self) -> usize {
        self.tracker.stagnation()
    }

    pub fn len(&self) -> usize {
        self.population.len()
    }

    pub fn mascot(&self) -> &Phenotype<C> {
        &self.mascot
    }

    pub fn score(&self) -> Option<&Score> {
        self.score.as_ref()
    }

    pub fn age(&self, current: usize) -> usize {
        current - self.generation
    }

    pub fn generation(&self) -> usize {
        self.generation
    }

    pub fn update_score(&mut self, score: Score, objective: &Objective)
    where
        C: PartialEq,
    {
        objective.sort(&mut self.population);

        self.score = Some(score);
        self.tracker
            .update(self.population[0].score().unwrap(), objective);
    }
}

impl<C: Chromosome> Scored for Species<C> {
    fn score(&self) -> Option<&Score> {
        self.score.as_ref()
    }
}

impl<C: Chromosome + Clone> Clone for Species<C> {
    fn clone(&self) -> Self {
        Species {
            id: self.id,
            generation: self.generation,
            tracker: self.tracker.clone(),
            score: self.score.clone(),
            mascot: self.mascot.clone(),
            population: self.population.clone(),
        }
    }
}

impl<C: Chromosome + PartialEq> PartialOrd for Species<C> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.score.as_ref().partial_cmp(&other.score.as_ref())
    }
}
impl<C: Chromosome + PartialEq> PartialEq for Species<C> {
    fn eq(&self, other: &Self) -> bool {
        self.score() == other.score()
            && self.id == other.id
            && self.mascot() == other.mascot()
            && self.len() == other.len()
            && self.stagnation() == other.stagnation()
            && self.generation == other.generation
    }
}

impl<C: Chromosome> Debug for Species<C> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "Species {{ members: {:?}, score: {:?}, best_score: {:?}, stagnation: {:?}, generation: {:?}, id: {:?} }}",
            self.len(),
            self.score,
            self.tracker.current(),
            self.tracker.stagnation(),
            self.generation,
            self.id
        )
    }
}
