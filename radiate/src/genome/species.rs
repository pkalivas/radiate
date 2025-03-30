use crate::{Chromosome, Objective, Phenotype, Population, Score};
use std::{
    fmt::{self, Debug, Formatter},
    sync::atomic::{AtomicU64, Ordering},
};

use super::PopulationView;

static ID_COUNTER: AtomicU64 = AtomicU64::new(1);

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SpeciesId(u64);

impl SpeciesId {
    pub fn new() -> Self {
        SpeciesId(ID_COUNTER.fetch_add(1, Ordering::SeqCst))
    }
}

pub struct Species<C: Chromosome> {
    mascot: Phenotype<C>,
    population: Population<C>,
    score: Score,
    best_score: Score,
    stagnation: usize,
    count: usize,
    id: SpeciesId,
    generation: usize,
}

impl<C: Chromosome> Species<C> {
    pub fn new(mascot: Phenotype<C>, generation: usize) -> Self {
        let score = mascot.score().unwrap();
        Species {
            mascot: Phenotype::clone(&mascot),
            population: Population::new(vec![mascot]),
            score: score.clone(),
            generation,
            count: 0,
            best_score: score.clone(),
            stagnation: 0,
            id: SpeciesId::new(),
        }
    }

    pub fn set_mascot(&mut self, mascot: Phenotype<C>) {
        self.mascot = mascot;
    }

    pub fn mascot(&self) -> &Phenotype<C> {
        &self.mascot
    }

    pub fn len(&self) -> usize {
        self.population.len()
    }

    pub fn set_score(&mut self, score: Score) {
        self.score = score;
    }

    pub fn id(&self) -> SpeciesId {
        self.id
    }

    pub fn stagnation(&self) -> usize {
        self.stagnation
    }

    pub fn score(&self) -> &Score {
        &self.score
    }

    pub fn add_member(&mut self, phenotype: &Phenotype<C>) {
        let new_phenotype = Phenotype::clone(phenotype);
        self.population.push(new_phenotype);
    }

    pub fn sort_by(&mut self, objective: &Objective) {
        objective.sort(&mut self.population);
    }

    pub fn population(&self) -> &Population<C> {
        &self.population
    }

    pub fn population_mut(&mut self) -> &mut Population<C> {
        &mut self.population
    }

    pub fn adjusted_scores(&self) -> Vec<Score> {
        self.population
            .get_scores()
            .iter()
            .map(|score| score.clone() / self.count as f32)
            .collect()
    }

    pub fn update_score(&mut self, score: Score, top_score: Score, objective: &Objective) {
        self.score = score.clone();
        if objective.is_better(&top_score, &self.best_score) {
            self.best_score = top_score;
            self.stagnation = 0;
        } else {
            self.stagnation += 1;
        }
    }

    pub fn age(&self, generation: usize) -> usize {
        generation - self.generation
    }
}

impl<C: Chromosome> Clone for Species<C> {
    fn clone(&self) -> Self {
        Species {
            mascot: Phenotype::clone(&self.mascot),
            population: self
                .population
                .iter()
                .map(|phenotype| Phenotype::clone(phenotype))
                .collect::<Population<C>>(),
            score: self.score.clone(),
            best_score: self.best_score.clone(),
            stagnation: self.stagnation,
            count: self.count,
            id: self.id,
            generation: self.generation,
        }
    }
}

impl<C: Chromosome> PopulationView<C> for Species<C> {
    fn push(&mut self, individual: Phenotype<C>) {
        self.population.push(individual);
        self.count += 1;
    }

    fn clear(&mut self) {
        self.population.clear();
        self.count = 0;
    }

    fn sort_by<F>(&mut self, f: F)
    where
        F: FnMut(&Phenotype<C>, &Phenotype<C>) -> std::cmp::Ordering,
    {
        self.population.sort_by(f);
    }
}

impl<C: Chromosome> AsRef<[Phenotype<C>]> for Species<C> {
    fn as_ref(&self) -> &[Phenotype<C>] {
        self.population.as_ref()
    }
}

impl<C: Chromosome> AsMut<[Phenotype<C>]> for Species<C> {
    fn as_mut(&mut self) -> &mut [Phenotype<C>] {
        self.population.as_mut()
    }
}

impl<C: Chromosome> Debug for Species<C> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "Species {{ members: {:?}, score: {:?}, best_score: {:?}, stagnation: {:?}, id: {:?} }}",
            self.len(),
            self.score,
            self.best_score,
            self.stagnation,
            self.id
        )
    }
}
