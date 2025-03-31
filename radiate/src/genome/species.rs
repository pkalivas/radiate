use crate::{Chromosome, Objective, Phenotype, Population, Score, Scored};
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

pub struct StangationTracker {
    current_score: Score,
    count: usize,
}

impl StangationTracker {
    pub fn new() -> Self {
        StangationTracker {
            current_score: Score::default(),
            count: 0,
        }
    }

    pub fn update(&mut self, new_score: &Score, objective: &Objective) {
        if objective.is_better(new_score, &self.current_score) {
            self.current_score = new_score.clone();
            self.count = 0;
            return;
        } else {
            self.count += 1;
        }
    }

    pub fn current_score(&self) -> &Score {
        &self.current_score
    }

    pub fn stagnation_count(&self) -> usize {
        self.count
    }
}

pub struct Species<C: Chromosome> {
    id: SpeciesId,
    mascot: Phenotype<C>,
    population: Population<C>,
    score: Score,
    stagnation_tracker: StangationTracker,
    generation: usize,
}

impl<C: Chromosome> Species<C> {
    pub fn new(mascot: Phenotype<C>, generation: usize) -> Self {
        let score = mascot.score().unwrap();
        Species {
            id: SpeciesId::new(),
            mascot: Phenotype::clone(&mascot),
            population: Population::new(vec![mascot]),
            score: score.clone(),
            stagnation_tracker: StangationTracker {
                current_score: score.clone(),
                count: 0,
            },

            generation,
        }
    }

    pub fn population(&self) -> &Population<C> {
        &self.population
    }

    pub fn population_mut(&mut self) -> &mut Population<C> {
        &mut self.population
    }

    pub fn set_mascot(&mut self, mascot: Phenotype<C>) {
        self.mascot = mascot;
    }

    pub fn mascot(&self) -> &Phenotype<C> {
        &self.mascot
    }

    pub fn set_score(&mut self, score: Score) {
        self.score = score;
    }

    pub fn id(&self) -> SpeciesId {
        self.id
    }

    pub fn stagnation(&self) -> usize {
        self.stagnation_tracker.count
    }

    pub fn stagnation_tracker(&self) -> &StangationTracker {
        &self.stagnation_tracker
    }

    pub fn generation(&self) -> usize {
        self.generation
    }

    pub fn score(&self) -> &Score {
        &self.score
    }

    pub fn len(&self) -> usize {
        self.population.len()
    }

    pub fn add_member(&mut self, phenotype: &Phenotype<C>) {
        let new_phenotype = Phenotype::clone(phenotype);
        self.population.push(new_phenotype);
    }

    pub fn update_score(&mut self, score: Score, objective: &Objective) {
        objective.sort(&mut self.population);

        self.score = score.clone();
        self.stagnation_tracker
            .update(&self.population.get(0).score().unwrap(), objective);
    }

    pub fn age(&self, generation: usize) -> usize {
        generation - self.generation
    }
}

impl<C: Chromosome> Clone for Species<C> {
    fn clone(&self) -> Self {
        Species {
            id: self.id,
            mascot: Phenotype::clone(&self.mascot),
            population: self
                .population
                .iter()
                .map(|phenotype| Phenotype::clone(phenotype))
                .collect::<Population<C>>(),
            score: self.score.clone(),
            stagnation_tracker: StangationTracker {
                current_score: self.stagnation_tracker.current_score.clone(),
                count: self.stagnation_tracker.count,
            },
            generation: self.generation,
        }
    }
}

impl<C: Chromosome> Scored for Species<C> {
    fn values(&self) -> impl AsRef<[f32]> {
        self.score.as_ref()
    }
    fn score(&self) -> Option<Score> {
        Some(self.score.clone())
    }
}

impl<C: Chromosome + PartialEq> PartialEq for Species<C> {
    fn eq(&self, other: &Self) -> bool {
        self.score == other.score
            && self.id == other.id
            && self.mascot() == other.mascot()
            && self.len() == other.len()
            && self.stagnation() == other.stagnation()
            && self.generation == other.generation
    }
}

impl<C: Chromosome> PartialOrd for Species<C> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.score.as_ref().partial_cmp(other.score.as_ref())
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
            "Species {{ members: {:?}, score: {:?}, best_score: {:?}, stagnation: {:?}, generation: {:?}, id: {:?} }}",
            self.len(),
            self.score,
            self.stagnation_tracker.current_score,
            self.stagnation_tracker.count,
            self.generation,
            self.id
        )
    }
}
