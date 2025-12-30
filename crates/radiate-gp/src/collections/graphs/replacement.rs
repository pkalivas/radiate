use super::GraphChromosome;
use crate::Factory;
use radiate_core::{Genotype, Population, ReplacementStrategy, random_provider};
use std::sync::Arc;

pub struct GraphReplacement;

/// A replacement strategy specialized for [GraphChromosome]s. [GraphChromosome]s differ form
/// most other chromosomes in that they grow in size over time. Because of this seemingly simple fact,
/// if we were to use a standard replacement strategy (like random replacement), we could end up with
/// a population where some individuals are exactly the same - completely ignoring the growth aspect of the
/// graphs. To negat this, this replacement strategy creates a new instance of the individual by keeping it's
/// structure, but resetting it's internal nodes with random new values. This way, we can keep evolved structure
/// of a graph, while still introducing new genetic material into the population.
///
/// Keep in mind this only gets called during the FilterStep of
/// evolution when a [Phenotype] is being replaced because of age or invalid state.
/// Thats to say, its rare.
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
