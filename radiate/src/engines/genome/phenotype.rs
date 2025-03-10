use crate::Chromosome;
use crate::engines::objectives::Score;

use super::{Valid, genotype::Genotype};

/// A `Phenotype` is a representation of an individual in the population. It contains:
/// * `Genotype` - the genetic representation of the individual
/// * `Score` - the score (fitness) of the individual as calculated by the fitness function
/// * `Generation` - the generation in which the individual was created
///
/// The `Phenotype` is a wrapper around the `Genotype` that adds additional information about the individual.
/// In traditional (biological) genetics, a phenotype is "the set of observable characteristics of an individual resulting
/// from the interaction of its genotype with the environment". As such, the `Phenotype` is the 'observable' part of the
/// individual (`Genotype`) that is being evolved by the genetic algorithm, hence the `Score` and `Generation` fields.
/// This allows the `Phenotype` to be sorted and compared based on the fitness (`Score`) of the individual (`Genotype`)
///
/// # Type Parameters
/// - `C`: The type of chromosome used in the genotype, which must implement the `Chromosome` trait.
///
#[derive(Clone, PartialEq, Debug)]
pub struct Phenotype<C: Chromosome> {
    pub genotype: Option<Genotype<C>>,
    pub score: Option<Score>,
    pub generation: usize,
}

impl<C: Chromosome> Phenotype<C> {
    /// Create a new instance of the `Phenotype` with the given `Genotype` and generation.
    pub fn from_genotype(genotype: Genotype<C>, generation: usize) -> Self {
        Phenotype {
            genotype: Some(genotype),
            score: None,
            generation,
        }
    }

    /// This is a convenience method that allows you to create a `Phenotype` from a list of chromosomes.
    /// Without it, we end up neededing to create a list of `Genes` then a list of `Chromosomes` then a `Genotype`,
    /// its just a lot. This method allows you to create a `Phenotype` from a list of chromosomes directly.
    pub fn from_chromosomes(chromosomes: Vec<C>, generation: usize) -> Self {
        Phenotype {
            genotype: Some(Genotype::new(chromosomes)),
            score: None,
            generation,
        }
    }

    pub fn genotype(&self) -> &Genotype<C> {
        self.genotype.as_ref().unwrap()
    }

    pub fn genotype_mut(&mut self) -> &mut Genotype<C> {
        self.genotype.as_mut().unwrap()
    }

    pub fn take_genotype(&mut self) -> Genotype<C> {
        self.score = None;
        self.genotype.take().unwrap()
    }

    pub fn set_genotype(&mut self, genotype: Genotype<C>) {
        self.genotype = Some(genotype);
    }

    pub fn set_score(&mut self, score: Option<Score>) {
        self.score = score;
    }

    pub fn score(&self) -> Option<&Score> {
        match &self.score {
            Some(score) => Some(score),
            None => None,
        }
    }

    /// Get the age of the individual in generations. The age is calculated as the
    /// difference between the given generation and the generation in which the individual was created.
    pub fn age(&self, generation: usize) -> usize {
        generation - self.generation
    }
}

/// Implement the `Valid` trait for the `Phenotype`. This allows the `Phenotype` to be checked for validity.
/// A `Phenotype` is valid if the `Genotype` is valid. The `GeneticEngine` checks the validity of the `Phenotype`
/// and will remove any invalid individuals from the population, replacing them with new individuals at the given generation.
impl<C: Chromosome> Valid for Phenotype<C> {
    fn is_valid(&self) -> bool {
        self.genotype.as_ref().unwrap().is_valid()
    }
}

impl<C: Chromosome> AsRef<Score> for Phenotype<C> {
    fn as_ref(&self) -> &Score {
        self.score.as_ref().unwrap()
    }
}

impl<C: Chromosome> AsRef<[f32]> for Phenotype<C> {
    fn as_ref(&self) -> &[f32] {
        self.score.as_ref().unwrap().as_ref()
    }
}

/// Implement the `PartialOrd` trait for the `Phenotype`. This allows the `Phenotype` to be compared
/// with other `Phenotype` instances. The comparison is based on the `Score` (fitness) of the `Phenotype`.
impl<C: Chromosome> PartialOrd for Phenotype<C> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.score.partial_cmp(&other.score)
    }
}
