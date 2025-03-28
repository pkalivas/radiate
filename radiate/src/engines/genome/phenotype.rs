use std::ops::{Deref, DerefMut};
use std::sync::{Arc, RwLock, RwLockReadGuard};

use super::{Valid, genotype::Genotype};
use crate::Chromosome;
use crate::engines::objectives::Score;

#[derive(Clone, PartialEq, Debug)]
pub struct PhenotypeCell<C: Chromosome> {
    genotype: Genotype<C>,
    score: Option<Score>,
    generation: usize,
    species_id: Option<u64>,
}

pub struct GenotypeGuard<'a, C: Chromosome> {
    inner: RwLockReadGuard<'a, PhenotypeCell<C>>,
}

impl<'a, C: Chromosome> GenotypeGuard<'a, C> {
    pub fn new(inner: &'a RwLock<PhenotypeCell<C>>) -> Self {
        let guard = inner.read().unwrap();
        GenotypeGuard { inner: guard }
    }
}

impl<'a, C: Chromosome> Deref for GenotypeGuard<'a, C> {
    type Target = Genotype<C>;

    fn deref(&self) -> &Self::Target {
        // Return the genotype from the inner cell
        &self.inner.genotype
    }
}

pub struct GenotypeGuardMut<'a, C: Chromosome> {
    inner: std::sync::RwLockWriteGuard<'a, PhenotypeCell<C>>,
}

impl<'a, C: Chromosome> GenotypeGuardMut<'a, C> {
    pub fn new(inner: &'a RwLock<PhenotypeCell<C>>) -> Self {
        let guard = inner.write().unwrap();
        GenotypeGuardMut { inner: guard }
    }
}

impl<'a, C: Chromosome> Deref for GenotypeGuardMut<'a, C> {
    type Target = Genotype<C>;

    fn deref(&self) -> &Self::Target {
        // Return the genotype from the inner cell
        &self.inner.genotype
    }
}

impl<'a, C: Chromosome> DerefMut for GenotypeGuardMut<'a, C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // Return the mutable genotype from the inner cell
        &mut self.inner.genotype
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
#[derive(Clone, Debug)]
pub struct Phenotype<C: Chromosome> {
    inner: Arc<RwLock<PhenotypeCell<C>>>,
}

impl<C: Chromosome> Phenotype<C> {
    pub fn genotype(&self) -> GenotypeGuard<C> {
        // Create a read guard to access the inner cell
        let guard = GenotypeGuard::new(&self.inner);
        guard
    }

    pub fn genotype_mut(&mut self) -> GenotypeGuardMut<C> {
        // Create a mutable guard to access the inner cell
        let guard = GenotypeGuardMut::new(&self.inner);
        guard
    }

    pub fn generation(&self) -> usize {
        // Lock the inner cell to access the generation
        let cell = self.inner.read().unwrap();
        cell.generation
    }

    // pub fn take_genotype(&mut self) -> Genotype<C> {
    //     self.score = None;
    //     self.genotype.take().unwrap()
    // }

    pub fn species_id(&self) -> Option<u64> {
        // Lock the inner cell to access the species_id
        let cell = self.inner.read().unwrap();
        cell.species_id
    }

    // pub fn set_genotype(&mut self, genotype: Genotype<C>) {
    //     self.genotype = Some(genotype);
    // }

    pub fn set_score(&mut self, score: Option<Score>) {
        // Lock the inner cell to set the score
        let mut cell = self.inner.write().unwrap();
        cell.score = score;
    }

    pub fn set_species_id(&mut self, species_id: Option<u64>) {
        // Lock the inner cell to set the species_id
        let mut cell = self.inner.write().unwrap();
        cell.species_id = species_id;
    }

    pub fn score(&self) -> ScoreGuard<C> {
        ScoreGuard::new(&self.inner)
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

impl<C: Chromosome> AsRef<Phenotype<C>> for &Phenotype<C> {
    fn as_ref(&self) -> &Phenotype<C> {
        self
    }
}

/// Implement the `PartialEq` trait for the `Phenotype`. This allows the `Phenotype` to be compared
/// with other `Phenotype` instances. The comparison is based on the `Score` (fitness) of the `Phenotype`.
impl<C: Chromosome> PartialEq for Phenotype<C> {
    fn eq(&self, other: &Self) -> bool {
        return self.genotype() == other.genotype()
            && self.score() == other.score()
            && self.generation() == other.generation()
            && self.species_id() == other.species_id();
    }
}

/// Implement the `PartialOrd` trait for the `Phenotype`. This allows the `Phenotype` to be compared
/// with other `Phenotype` instances. The comparison is based on the `Score` (fitness) of the `Phenotype`.
impl<C: Chromosome> PartialOrd for Phenotype<C> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.score.partial_cmp(&other.score)
    }
}

impl<C: Chromosome> From<(Genotype<C>, usize, Option<u64>)> for Phenotype<C> {
    fn from((genotype, generation, species_id): (Genotype<C>, usize, Option<u64>)) -> Self {
        Phenotype {
            genotype: Some(genotype),
            score: None,
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
            genotype: Some(Genotype::new(chromosomes)),
            score: None,
            generation,
            species_id: None,
        }
    }
}
