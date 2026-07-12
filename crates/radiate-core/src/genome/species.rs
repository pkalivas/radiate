use super::{Chromosome, Phenotype};
use crate::{Objective, Score, objectives::Scored, phenotype::PhenotypeId, tracker::Tracker};
use radiate_utils::sentry_id;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    fmt::{self, Debug, Formatter},
    sync::atomic::AtomicU64,
};

sentry_id!(SpeciesId);

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Species<C: Chromosome> {
    pub id: SpeciesId,
    pub generation: usize,
    pub tracker: Tracker<Score>,
    pub adjusted_score: Option<Score>,
    pub mascot: Phenotype<C>,
    pub size: usize,
    pub members: HashSet<PhenotypeId>,
}

impl<C: Chromosome> Species<C> {
    pub fn new(generation: usize, initial: Phenotype<C>) -> Self {
        Species {
            id: SpeciesId::new(),
            generation,
            tracker: Tracker::new(),
            adjusted_score: Some(initial.score().unwrap().clone()),
            mascot: initial,
            size: 0,
            members: HashSet::new(),
        }
    }

    pub fn id(&self) -> SpeciesId {
        self.id
    }

    pub fn add_member(&mut self, phenotype_id: PhenotypeId) {
        self.size += 1;
        self.members.insert(phenotype_id);
    }

    pub fn stagnation(&self) -> usize {
        self.tracker.stagnation()
    }

    pub fn len(&self) -> usize {
        self.size
    }

    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    pub fn set_new_mascot(&mut self, mascot: Phenotype<C>) {
        self.mascot = mascot;
        self.size = 0;
        self.members.clear();
    }

    pub fn mascot(&self) -> &Phenotype<C> {
        &self.mascot
    }

    pub fn adj_score(&self) -> Option<&Score> {
        self.adjusted_score.as_ref()
    }

    pub fn raw_score(&self) -> Option<&Score> {
        self.tracker.best()
    }

    pub fn age(&self, current: usize) -> usize {
        current - self.generation
    }

    pub fn generation(&self) -> usize {
        self.generation
    }

    pub fn update_score(&mut self, raw_score: Score, adjusted_score: Score, objective: &Objective)
    where
        C: PartialEq,
    {
        self.adjusted_score = Some(adjusted_score);
        self.tracker.update(&raw_score, objective);
    }
}

impl<C: Chromosome> Scored for Species<C> {
    fn score(&self) -> Option<&Score> {
        self.adjusted_score.as_ref()
    }
}

impl<C: Chromosome + Clone> Clone for Species<C> {
    fn clone(&self) -> Self {
        Species {
            id: self.id,
            generation: self.generation,
            tracker: self.tracker.clone(),
            adjusted_score: self.adjusted_score.clone(),
            mascot: self.mascot.clone(),
            size: self.size,
            members: self.members.clone(),
        }
    }
}

impl<C: Chromosome + PartialEq> PartialOrd for Species<C> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.adjusted_score
            .as_ref()
            .partial_cmp(&other.adjusted_score.as_ref())
    }
}
impl<C: Chromosome + PartialEq> PartialEq for Species<C> {
    fn eq(&self, other: &Self) -> bool {
        self.adj_score() == other.adj_score()
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
            "id={}  size={:>2}  adj={}  raw={}  stag={}  gen={}",
            self.id.get(),
            self.len(),
            fmt_opt_score(self.adjusted_score.as_ref()),
            fmt_opt_score(self.tracker.current()),
            self.tracker.stagnation(),
            self.generation,
        )
    }
}

fn fmt_opt_score(score: Option<&Score>) -> String {
    match score {
        Some(s) if s.is_single_objective() => format!("{:.6}", s.first().unwrap_or(f32::NAN)),
        Some(s) => format!("{:?}", s.as_slice()),
        None => "?".to_string(),
    }
}
