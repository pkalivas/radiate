use super::{Chromosome, EngineError, Genotype, Phenotype, Population, random_provider};
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
        encoder: Arc<dyn Fn() -> Result<Genotype<C>, EngineError>>,
    ) -> Result<(), EngineError>;
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
        encoder: Arc<dyn Fn() -> Result<Genotype<C>, EngineError>>,
    ) -> Result<(), EngineError> {
        let genotype = encoder()?;
        population[replace_idx] = Phenotype::from_genotype(genotype, generation);

        Ok(())
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
        _: Arc<dyn Fn() -> Result<Genotype<C>, EngineError>>,
    ) -> Result<(), EngineError> {
        let random_member = random_provider::random_range(0..population.len());
        let new_phenotype = population[random_member].genotype().clone();
        population[replace_idx] = Phenotype::from_genotype(new_phenotype, generation);

        Ok(())
    }
}
