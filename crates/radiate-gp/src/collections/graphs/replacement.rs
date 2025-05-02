use super::GraphChromosome;
use crate::Factory;
use radiate_core::{Genotype, Population, ReplacementStrategy, random_provider};
use std::sync::Arc;

pub struct GraphReplacement;

impl<T> ReplacementStrategy<GraphChromosome<T>> for GraphReplacement
where
    T: Clone + PartialEq + Default,
{
    fn replace(
        &self,
        population: &Population<GraphChromosome<T>>,
        _: Arc<dyn Fn() -> Genotype<GraphChromosome<T>> + Send + Sync>,
    ) -> Genotype<GraphChromosome<T>> {
        let random_member = random_provider::range(0..population.len());
        Genotype::from(
            population[random_member]
                .genotype()
                .iter()
                .map(|chromosome| chromosome.new_instance(None))
                .collect::<Vec<GraphChromosome<T>>>(),
        )
    }
}
