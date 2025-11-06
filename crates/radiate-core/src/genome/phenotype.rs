use super::{Valid, genotype::Genotype};
use crate::Result;
use crate::objectives::Score;
use crate::{Chromosome, objectives::Scored};
use radiate_error::radiate_err;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::hash::Hash;
use std::sync::atomic::{AtomicU64, Ordering};

/// A unique identifier for a [Phenotype]. This is used to identify the [Phenotype] in the population.
/// It is a simple wrapper around a `u64` value. Using this, we can uniquely identify each [Phenotype]
/// and can track them by a sort of 'version'. Every time a [Phenotype] is created or invalidated, its ID is updated.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[repr(transparent)]
pub struct PhenotypeId(pub u64);

impl PhenotypeId {
    pub fn new() -> Self {
        static PHENOTYPE_ID: AtomicU64 = AtomicU64::new(0);
        PhenotypeId(PHENOTYPE_ID.fetch_add(1, Ordering::Relaxed))
    }
}

/// A [Phenotype] is a representation of an individual in the population. It contains:
/// * `Genotype` - the genetic representation of the individual
/// * `Score` - the score (fitness) of the individual as calculated by the fitness function
/// * `Generation` - the generation in which the individual was created
/// * `id` - a unique identifier for the [Phenotype]
///
/// The [Phenotype] is a wrapper around the [Genotype] that adds additional information about the individual.
/// In traditional (biological) genetics, a phenotype is "the set of observable characteristics of an individual resulting
/// from the interaction of its genotype with the environment". As such, the [Phenotype] is the 'observable' part of the
/// individual ([Genotype]) that is being evolved by the genetic algorithm, hence the [Score] and generation fields.
/// This allows the [Phenotype] to be sorted and compared based on the fitness ([Score]) of the individual ([Genotype])
///
/// # Type Parameters
/// - `C`: The type of chromosome used in the genotype, which must implement the `Chromosome` trait.
///
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Phenotype<C: Chromosome> {
    genotype: Option<Genotype<C>>,
    score: Option<Score>,
    generation: usize,
    id: PhenotypeId,
}

impl<C: Chromosome> Phenotype<C> {
    pub fn genotype(&self) -> &Genotype<C> {
        match &self.genotype {
            Some(genotype) => genotype,
            None => panic!("Genotype is None - this shouldn't happen."),
        }
    }

    pub fn genotype_mut(&mut self) -> &mut Genotype<C> {
        match &mut self.genotype {
            Some(genotype) => genotype,
            None => panic!("Genotype mut is None - this shouldn't happen."),
        }
    }

    pub fn take_genotype(&mut self) -> Result<Genotype<C>> {
        self.genotype.take().map_or(
            Err(radiate_err!(Genome: "Genotype is None - this shouldn't happen.")),
            Ok,
        )
    }

    pub fn set_genotype(&mut self, genotype: Genotype<C>) {
        self.genotype = Some(genotype);
    }

    pub fn set_score(&mut self, score: Option<Score>) {
        self.score = score;
    }

    pub fn generation(&self) -> usize {
        self.generation
    }

    pub fn score(&self) -> Option<&Score> {
        self.score.as_ref()
    }

    pub fn id(&self) -> PhenotypeId {
        self.id
    }

    pub fn invalidate(&mut self, generation: usize) {
        if self.score.is_none() && self.generation == generation {
            return;
        }

        self.score = None;
        self.generation = generation;
        self.id = PhenotypeId::new();
    }

    /// Get the age of the individual in generations. The age is calculated as the
    /// difference between the given generation and the generation in which the individual was created.
    pub fn age(&self, generation: usize) -> usize {
        generation - self.generation
    }
}

impl<C: Chromosome> Scored for Phenotype<C> {
    fn score(&self) -> Option<&Score> {
        self.score.as_ref()
    }
}

/// Implement the [Valid] trait for the `Phenotype`. This allows the `Phenotype` to be checked for validity.
/// A `Phenotype` is valid if the `Genotype` is valid. The `GeneticEngine` checks the validity of the `Phenotype`
/// and will remove any invalid individuals from the population, replacing them with new individuals at the given generation.
impl<C: Chromosome> Valid for Phenotype<C> {
    fn is_valid(&self) -> bool {
        self.genotype().is_valid()
    }
}

/// Implement the `AsRef<[f32]>` trait for the `Phenotype`. This allows the `Phenotype` to be converted to a slice of `f32`
/// which will be the `Score` of the `Phenotype`. This is used when adding a `Phenotype` to a pareto `Front` for sorting.
impl<C: Chromosome> AsRef<[f32]> for Phenotype<C> {
    fn as_ref(&self) -> &[f32] {
        self.score().unwrap().as_ref()
    }
}

/// Implement the `PartialOrd` trait for the [Phenotype]. This allows the [Phenotype] to be compared
/// with other `Phenotype` instances. The comparison is based on the `Score` (fitness) of the `Phenotype`.
impl<C> PartialOrd for Phenotype<C>
where
    C: Chromosome + PartialEq,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let self_score = self.score();
        let other_score = other.score();

        self_score.partial_cmp(&other_score)
    }
}

impl<C> Eq for Phenotype<C> where C: Chromosome + PartialEq {}

impl<C: Chromosome> Hash for Phenotype<C> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl<C: Chromosome> From<Genotype<C>> for Phenotype<C> {
    fn from(genotype: Genotype<C>) -> Self {
        Phenotype {
            genotype: Some(genotype),
            score: None,
            generation: 0,
            id: PhenotypeId::new(),
        }
    }
}

impl<C: Chromosome> From<(Genotype<C>, usize)> for Phenotype<C> {
    fn from((genotype, generation): (Genotype<C>, usize)) -> Self {
        Phenotype {
            genotype: Some(genotype),
            score: None,
            generation,
            id: PhenotypeId::new(),
        }
    }
}

/// This is a convenience method that allows you to create a [Phenotype] from a list of [Chromosome]s.
/// Without it, we end up neededing to create a list of [Gene](super::chromosomes::Gene)s
/// then a list of [Chromosome]s then a [Genotype], its just a lot.
/// This method allows you to create a [Phenotype] from a list of chromosomes directly.
impl<C: Chromosome> From<(Vec<C>, usize)> for Phenotype<C> {
    fn from((chromosomes, generation): (Vec<C>, usize)) -> Self {
        Phenotype {
            genotype: Some(Genotype::new(chromosomes)),
            score: None,
            generation,
            id: PhenotypeId::new(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{FloatChromosome, FloatGene};

    #[test]
    fn test_phenotype_creation() {
        let phenotype = Phenotype::from((
            vec![
                FloatChromosome::from(FloatGene::from(0.0..5.0)),
                FloatChromosome::from(FloatGene::from(5.0..10.0)),
            ],
            0,
        ));

        assert_eq!(phenotype.generation(), 0);
        assert!(phenotype.score().is_none());
        assert!(phenotype.is_valid());
    }

    #[test]
    fn test_phenotype_age() {
        let genotype = Genotype::from(FloatChromosome::from(FloatGene::from(0.0..5.0)));
        let phenotype = Phenotype::from((genotype, 5));

        assert_eq!(phenotype.age(10), 5);
    }

    #[test]
    #[cfg(feature = "serde")]
    fn test_phenotype_can_serialize() {
        let genotype = Genotype::from(FloatChromosome::new(vec![FloatGene::from(0.0..5.0)]));
        let phenotype = Phenotype::from((genotype, 0));

        let serialized = serde_json::to_string(&phenotype).expect("Failed to serialize Phenotype");
        let deserialized: Phenotype<FloatChromosome> =
            serde_json::from_str(&serialized).expect("Failed to deserialize Phenotype");

        assert_eq!(phenotype, deserialized);
    }
}
