use crate::{Chromosome, Genotype, Objective, Score};
use std::{
    fmt::{self, Debug, Formatter},
    sync::atomic::{AtomicU64, Ordering},
};

static ID_COUNTER: AtomicU64 = AtomicU64::new(1);

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SpeciesId(u64);

impl SpeciesId {
    pub fn new() -> Self {
        SpeciesId(ID_COUNTER.fetch_add(1, Ordering::SeqCst))
    }
}

#[derive(Clone)]
pub struct Species<C: Chromosome> {
    mascot: Genotype<C>,
    score: Score,
    best_score: Score,
    stagnation: usize,
    count: usize,
    id: SpeciesId,
    generation: usize,
}

impl<C: Chromosome> Species<C> {
    pub fn new(mascot: Genotype<C>, score: Score, generation: usize) -> Self {
        Species {
            mascot,
            score: score.clone(),
            generation,
            count: 0,
            best_score: score.clone(),
            stagnation: 0,
            id: SpeciesId::new(),
        }
    }

    pub fn set_mascot(&mut self, mascot: Genotype<C>) {
        self.mascot = mascot;
    }

    pub fn mascot(&self) -> &Genotype<C> {
        &self.mascot
    }

    pub fn count(&self) -> usize {
        self.count
    }

    pub fn set_count(&mut self, count: usize) {
        self.count = count;
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

impl<C: Chromosome> Debug for Species<C> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "Species {{ members: {:?}, score: {:?}, best_score: {:?}, stagnation: {:?}, count: {:?}, id: {:?} }}",
            self.count, self.score, self.best_score, self.stagnation, self.count, self.id
        )
    }
}
