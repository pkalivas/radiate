use super::{Valid, genotype::Genotype};
use crate::objectives::Score;
use crate::sync::{RwCell, RwCellGuard, RwCellGuardMut};
use crate::{Chromosome, Scored, SpeciesId};
use std::ops::Deref;
use std::sync::atomic::AtomicU64;

static PHENOTYPE_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PhenotypeId(u64);

impl PhenotypeId {
    pub fn new() -> Self {
        PhenotypeId(PHENOTYPE_ID_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst))
    }
}

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
#[derive(Clone, Debug, PartialEq)]
pub struct Phenotype<C: Chromosome> {
    genotype: RwCell<Genotype<C>>,
    score: RwCell<Option<Score>>,
    id: PhenotypeId,
    generation: usize,
    species_id: Option<SpeciesId>,
}

impl<C: Chromosome> Phenotype<C> {
    pub fn clone(other: &Phenotype<C>) -> Self {
        Phenotype {
            id: other.id,
            genotype: RwCell::clone(&other.genotype),
            score: RwCell::clone(&other.score),
            generation: other.generation,
            species_id: other.species_id,
        }
    }

    pub fn id(&self) -> PhenotypeId {
        self.id
    }

    pub fn invalidate(&mut self, generation: usize) {
        self.score.set(None);
        self.generation = generation; // Update the generation to the current one
        self.id = PhenotypeId::new();
    }

    pub fn genotype(&self) -> RwCellGuard<Genotype<C>> {
        self.genotype.read()
    }

    pub fn genotype_mut(&mut self) -> RwCellGuardMut<Genotype<C>> {
        self.genotype.write()
    }

    pub fn generation(&self) -> usize {
        self.generation
    }

    pub fn set_generation(&mut self, generation: usize) {
        self.generation = generation;
    }

    pub fn species_id(&self) -> Option<SpeciesId> {
        self.species_id
    }

    pub fn set_score(&mut self, score: Option<Score>) {
        self.score.set(score);
    }

    pub fn set_species_id(&mut self, species_id: Option<SpeciesId>) {
        self.species_id = species_id;
    }

    pub fn score(&self) -> Option<Score> {
        let lock = self.score.read();
        lock.inner().clone()
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
        self.genotype().deref().is_valid()
    }
}

impl<C: Chromosome> AsRef<Phenotype<C>> for Phenotype<C> {
    fn as_ref(&self) -> &Phenotype<C> {
        self
    }
}

impl<C: Chromosome> Scored for Phenotype<C> {
    fn values(&self) -> impl AsRef<[f32]> {
        let score = self.score();
        if score.is_none() {
            return Score::default();
        }

        score.unwrap()
    }
    fn score(&self) -> Option<Score> {
        self.score()
    }
}

/// Implement the `PartialOrd` trait for the `Phenotype`. This allows the `Phenotype` to be compared
/// with other `Phenotype` instances. The comparison is based on the `Score` (fitness) of the `Phenotype`.
impl<C: Chromosome> PartialOrd for Phenotype<C> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let self_score = self.score();
        let other_score = other.score();

        self_score.partial_cmp(&other_score)
    }
}

impl<C: Chromosome> From<(Genotype<C>, usize, Option<SpeciesId>)> for Phenotype<C> {
    fn from((genotype, generation, species_id): (Genotype<C>, usize, Option<SpeciesId>)) -> Self {
        Phenotype {
            id: PhenotypeId::new(),
            genotype: RwCell::new(genotype),
            score: RwCell::new(None),
            generation,
            species_id,
        }
    }
}

/// This is a convenience method that allows you to create a `Phenotype` from a list of chromosomes.
/// Without it, we end up neededing to create a list of `Genes` then a list of `Chromosomes` then a `Genotype`,
/// its just a lot. This method allows you to create a `Phenotype` from a list of chromosomes directly.
impl<C: Chromosome> From<(Vec<C>, usize)> for Phenotype<C> {
    fn from((chromosomes, generation): (Vec<C>, usize)) -> Self {
        Phenotype {
            id: PhenotypeId::new(),
            genotype: RwCell::new(Genotype::new(chromosomes)),
            score: RwCell::new(None),
            generation,
            species_id: None,
        }
    }
}

impl<C: Chromosome> From<(&Phenotype<C>, Score)> for Phenotype<C> {
    fn from((phenotype, score): (&Phenotype<C>, Score)) -> Self {
        Phenotype {
            id: phenotype.id,
            genotype: RwCell::clone(&phenotype.genotype),
            score: RwCell::new(Some(score)),
            generation: phenotype.generation,
            species_id: phenotype.species_id,
        }
    }
}
