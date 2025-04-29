use super::{Chromosome, Phenotype, Population};
use crate::{Objective, Score, objectives::Scored, tracker::Tracker};
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(transparent)]
pub struct SpeciesId(u64);

impl SpeciesId {
    pub fn new() -> Self {
        static SPECIES_ID: AtomicU64 = AtomicU64::new(0);
        SpeciesId(SPECIES_ID.fetch_add(1, Ordering::SeqCst))
    }
}

#[derive(Debug, Clone)]
pub struct Species<C: Chromosome> {
    pub id: SpeciesId,
    pub generation: usize,
    pub tracker: Tracker<Score>,
    pub score: Option<Score>,
    pub mascot: Phenotype<C>,
    pub population: Population<C>,
}

impl<C: Chromosome> Species<C> {
    pub fn new(generation: usize, initial: &Phenotype<C>) -> Self {
        Species {
            id: SpeciesId::new(),
            generation,
            tracker: Tracker::new(),
            score: None,
            mascot: initial.clone(),
            population: Population::new(vec![initial.clone()]),
        }
    }

    pub fn push(&mut self, individual: &Phenotype<C>) {
        self.population.push(individual.clone());
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
        self.tracker.current()
    }

    pub fn update_score(&mut self, score: Score, objective: &Objective) {
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

impl<C: Chromosome> PartialOrd for Species<C> {
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
