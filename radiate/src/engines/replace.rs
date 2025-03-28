use super::{Chromosome, Genotype, Phenotype, Population, random_provider};
use std::sync::Arc;

/// Trait for replacement strategies in genetic algorithms.
///
/// This trait defines a method for replacing a member of the population with a new individual
/// after the current individual has been determined to be invalid. Typically, this is done by
/// replacing the individual with a new one generated by the encoder. But in some cases, it may
/// be desirable to replace the individual in a different way, such as by sampling from the
/// population.
pub trait ReplacementStrategy<C: Chromosome> {
    fn replace(
        &self,
        replace_idx: usize,
        generation: usize,
        population: &mut Population<C>,
        encoder: Arc<dyn Fn() -> Genotype<C>>,
    );
}

/// Replacement strategy that replaces the individual with a new one generated by the encoder.
/// This is the default replacement strategy used in genetic algorithms.
pub struct EncodeReplace;

impl<C: Chromosome> ReplacementStrategy<C> for EncodeReplace {
    fn replace(
        &self,
        replace_idx: usize,
        generation: usize,
        population: &mut Population<C>,
        encoder: Arc<dyn Fn() -> Genotype<C>>,
    ) {
        population[replace_idx] = Phenotype::from((encoder(), generation, None)).into();
    }
}

/// Replacement strategy that replaces the individual with a random member of the population.
/// This can be useful in cases where the population is large and diverse or when the
/// chromosome grows or changes in size, thus encoding a new individual can result
/// in a member that that lacks significant diversity.
pub struct PopulationSampleReplace;

impl<C: Chromosome> ReplacementStrategy<C> for PopulationSampleReplace {
    fn replace(
        &self,
        replace_idx: usize,
        generation: usize,
        population: &mut Population<C>,
        _: Arc<dyn Fn() -> Genotype<C>>,
    ) {
        let random_member = random_provider::range(0..population.len());
        let new_phenotype = population[random_member].read().genotype().clone();
        population[replace_idx] = Phenotype::from((new_phenotype, generation, None)).into();
    }
}
