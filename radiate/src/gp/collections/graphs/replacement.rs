use super::GraphChromosome;
use crate::Factory;
use crate::{Genotype, Population, ReplacementStrategy, random_provider};
use std::sync::Arc;

/// Replacement strategy for replacing `GraphChromosome` individuals in a population.
pub struct GraphReplacement;

/// This replacement strategy takes a random member of the population and
/// creates a new instance of the chromosome from it. Because the `GraphChromosome`
/// can grow during evolution, typical replacement strategies would create baseline
/// chromosomes that are not necessarily very useful as evolution progresses.
/// This strategy allows the population to sample from the population and create
/// new chromosomes with different alleles that are more likely to be useful
/// in the current generation.
impl<T> ReplacementStrategy<GraphChromosome<T>> for GraphReplacement
where
    T: Clone + PartialEq + Default,
{
    fn replace(
        &self,
        population: &Population<GraphChromosome<T>>,
        _: Arc<dyn Fn() -> Genotype<GraphChromosome<T>>>,
    ) -> Genotype<GraphChromosome<T>> {
        let random_member = random_provider::range(0..population.len());
        Genotype::from(
            population[random_member]
                .genotype()
                .iter()
                .map(|chromosomee| chromosomee.new_instance(None))
                .collect::<Vec<GraphChromosome<T>>>(),
        )
    }
}
