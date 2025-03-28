use super::{Valid, genotype::Genotype};
use crate::engines::objectives::Score;
use crate::{Chromosome, ScoreGuard, ScoreView};
use std::ops::Deref;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

#[derive(Debug)]
pub struct InnerPhenotype<C: Chromosome> {
    pub genotype: RwLock<Genotype<C>>,
    pub score: RwLock<Option<Score>>,
}

#[derive(Debug)]
pub struct GenotypeGuard<'a, C: Chromosome> {
    genotype: RwLockReadGuard<'a, Genotype<C>>,
}

impl<'a, C: Chromosome> GenotypeGuard<'a, C> {
    pub fn new(lock: &'a RwLock<Genotype<C>>) -> Self {
        GenotypeGuard {
            genotype: lock.read().unwrap(),
        }
    }
}

impl<'a, C: Chromosome> Deref for GenotypeGuard<'a, C> {
    type Target = Genotype<C>;

    fn deref(&self) -> &Self::Target {
        &self.genotype
    }
}

#[derive(Debug)]
pub struct GenotypeGuardMut<'a, C: Chromosome> {
    genotype: RwLockWriteGuard<'a, Genotype<C>>,
}

impl<'a, C: Chromosome> GenotypeGuardMut<'a, C> {
    pub fn new(lock: &'a RwLock<Genotype<C>>) -> Self {
        GenotypeGuardMut {
            genotype: lock.write().unwrap(),
        }
    }
}

impl<'a, C: Chromosome> Deref for GenotypeGuardMut<'a, C> {
    type Target = Genotype<C>;

    fn deref(&self) -> &Self::Target {
        &self.genotype
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
#[derive(Debug)]
pub struct Phenotype<C: Chromosome> {
    pub inner: Arc<InnerPhenotype<C>>,
    pub generation: usize,
    pub species_id: Option<u64>,
}

impl<C: Chromosome> Phenotype<C> {
    pub fn clone(other: &Self) -> Self {
        Phenotype {
            inner: Arc::clone(&other.inner),
            generation: other.generation,
            species_id: other.species_id,
        }
    }

    pub fn inner(&self) -> Arc<InnerPhenotype<C>> {
        self.inner.clone()
    }

    pub fn genotype(&self) -> GenotypeGuard<'_, C> {
        GenotypeGuard::new(&self.inner.genotype)
    }

    pub fn genotype_mut(&self) -> GenotypeGuardMut<'_, C> {
        GenotypeGuardMut::new(&self.inner.genotype)
    }

    pub fn species_id(&self) -> Option<u64> {
        self.species_id
    }

    pub fn set_score(&mut self, score: Option<Score>) {
        *self.inner.score.write().unwrap() = score;
    }

    pub fn set_species_id(&mut self, species_id: Option<u64>) {
        self.species_id = species_id;
    }

    pub fn set_generation(&mut self, generation: usize) {
        self.generation = generation;
    }

    pub fn score(&self) -> Option<ScoreGuard> {
        let score = self.inner.score.read().unwrap();
        if let Some(_) = *score {
            // If the score is Some, return a ScoreGuard to access the score.
            Some(ScoreGuard::new(self.inner.score.read().unwrap()))
        } else {
            // If the score is None, return None.
            None
        }
    }

    /// Get the age of the individual in generations. The age is calculated as the
    /// difference between the given generation and the generation in which the individual was created.
    pub fn age(&self, generation: usize) -> usize {
        generation - self.generation
    }
}

/// Implement the `ScoreView` trait for the `Phenotype`. This allows the `Phenotype` to be used as a view
/// for the `Score` of the individual. This is useful for sorting and comparing individuals based on their score.
impl<C: Chromosome> ScoreView for Phenotype<C> {
    fn score(&self) -> impl Deref<Target = Option<Score>> {
        self.inner.score.read().unwrap()
    }
}

/// Implement the `Valid` trait for the `Phenotype`. This allows the `Phenotype` to be checked for validity.
/// A `Phenotype` is valid if the `Genotype` is valid. The `GeneticEngine` checks the validity of the `Phenotype`
/// and will remove any invalid individuals from the population, replacing them with new individuals at the given generation.
impl<C: Chromosome> Valid for Phenotype<C> {
    fn is_valid(&self) -> bool {
        self.inner.genotype.read().unwrap().is_valid()
    }
}

impl<C: Chromosome> AsRef<Phenotype<C>> for &Phenotype<C> {
    fn as_ref(&self) -> &Phenotype<C> {
        self
    }
}

impl<C: Chromosome> Clone for Phenotype<C> {
    fn clone(&self) -> Self {
        Phenotype {
            inner: Arc::new(InnerPhenotype {
                genotype: RwLock::new(self.inner.genotype.read().unwrap().clone()),
                score: RwLock::new(self.inner.score.read().unwrap().clone()),
            }),
            generation: self.generation,
            species_id: self.species_id,
        }
    }
}

impl<C: Chromosome> PartialEq for Phenotype<C> {
    fn eq(&self, other: &Self) -> bool {
        *self.inner.genotype.read().unwrap() == *other.inner.genotype.read().unwrap()
            && *self.inner.score.read().unwrap() == *other.inner.score.read().unwrap()
            && self.generation == other.generation
            && self.species_id == other.species_id
    }
}

/// Implement the `PartialOrd` trait for the `Phenotype`. This allows the `Phenotype` to be compared
/// with other `Phenotype` instances. The comparison is based on the `Score` (fitness) of the `Phenotype`.
impl<C: Chromosome> PartialOrd for Phenotype<C> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let self_score = self.inner.score.read().unwrap();
        let other_score = other.inner.score.read().unwrap();

        self_score.partial_cmp(&other_score)
    }
}

impl<C: Chromosome> From<(Genotype<C>, usize, Option<u64>)> for Phenotype<C> {
    fn from((genotype, generation, species_id): (Genotype<C>, usize, Option<u64>)) -> Self {
        Phenotype {
            inner: Arc::new(InnerPhenotype {
                genotype: RwLock::new(genotype),
                score: RwLock::new(None),
            }),
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
            inner: Arc::new(InnerPhenotype {
                genotype: RwLock::new(Genotype::new(chromosomes)),
                score: RwLock::new(None),
            }),
            generation,
            species_id: None,
        }
    }
}

unsafe impl<C: Chromosome> Send for Phenotype<C> {}
unsafe impl<C: Chromosome> Sync for Phenotype<C> {}
