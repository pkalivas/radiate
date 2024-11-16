use crate::engines::score::Score;

use super::{genes::gene::Gene, genotype::Genotype};

pub struct Phenotype<G, A>
where
    G: Gene<G, A>,
{
    pub genotype: Genotype<G, A>,
    pub score: Option<Score>,
    pub generation: i32,
}

impl<G, A> Phenotype<G, A>
where
    G: Gene<G, A>,
{
    pub fn from_genotype(genotype: Genotype<G, A>, generation: i32) -> Self {
        Phenotype {
            genotype,
            score: None,
            generation,
        }
    }

    pub fn genotype(&self) -> &Genotype<G, A> {
        &self.genotype
    }

    pub fn genotype_mut(&mut self) -> &mut Genotype<G, A> {
        &mut self.genotype
    }

    pub fn score(&self) -> &Option<Score> {
        &self.score
    }

    pub fn set_score(&mut self, score: Option<Score>) {
        self.score = score;
    }

    pub fn age(&self, generation: i32) -> i32 {
        generation - self.generation
    }
}

impl<G, A> Clone for Phenotype<G, A>
where
    G: Gene<G, A>,
{
    fn clone(&self) -> Self {
        Phenotype {
            genotype: self.genotype.clone(),
            score: match &self.score {
                Some(score) => Some(score.clone()),
                None => None,
            },
            generation: self.generation,
        }
    }
}

impl<G, A> PartialEq for Phenotype<G, A>
where
    G: Gene<G, A>,
{
    fn eq(&self, other: &Self) -> bool {
        self.genotype == other.genotype
            && self.score == other.score
            && self.generation == other.generation
    }
}

impl<G, A> PartialOrd for Phenotype<G, A>
where
    G: Gene<G, A>,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.score.partial_cmp(&other.score)
    }
}

impl<G, A> std::fmt::Debug for Phenotype<G, A>
where
    G: Gene<G, A> + std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?}, generation: {:?}, score: {:?}",
            self.genotype, self.generation, self.score
        )
    }
}
