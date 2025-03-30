use super::EngineStep;
use crate::{
    Chromosome, GeneticEngineParams, Genotype, Population, ReplacementStrategy, Species,
    metric_names,
};
use crate::{Metric, Valid};
use std::sync::Arc;

pub struct FilterStep<C: Chromosome> {
    replacer: Arc<dyn ReplacementStrategy<C>>,
    encoder: Arc<dyn Fn() -> Genotype<C>>,
    max_age: usize,
}

/// Filters the population to remove individuals that are too old or invalid. The maximum age
/// of an individual is determined by the 'max_age' parameter in the genetic engine parameters.
/// If an individual's age exceeds this limit, it is replaced with a new individual. Similarly,
/// if an individual is found to be invalid (i.e., its genotype is not valid, provided by the `valid` trait),
/// it is replaced with a new individual. This method ensures that the population remains
/// healthy and that only valid individuals are allowed to reproduce or survive to the next generation.
///
/// The method in which a new individual is created is determined by the `filter_strategy`
/// parameter in the genetic engine parameters and is either `FilterStrategy::Encode` or
/// `FilterStrategy::PopulationSample`. If the `FilterStrategy` is `FilterStrategy::Encode`, then a new individual
/// is created using the `encode` method of the `Problem` trait, while if the `FilterStrategy`
/// is `FilterStrategy::PopulationSample`, then a new individual is created by randomly selecting
/// an individual from the population.
impl<C, T> EngineStep<C, T> for FilterStep<C>
where
    C: Chromosome + 'static,
    T: Clone + Send + 'static,
{
    fn register(params: &GeneticEngineParams<C, T>) -> Option<Box<Self>>
    where
        Self: Sized,
    {
        let replacement_strategy = params.replacement_strategy();
        let problem = params.problem();
        let encoder = Arc::new(move || problem.encode());

        Some(Box::new(FilterStep {
            replacer: Arc::clone(&replacement_strategy),
            encoder,
            max_age: params.max_age(),
        }))
    }

    fn execute(
        &self,
        generation: usize,
        population: &mut Population<C>,
        species: &mut Vec<Species<C>>,
    ) -> Vec<Metric> {
        let mut age_count = 0_f32;
        let mut invalid_count = 0_f32;
        for i in 0..population.len() {
            let phenotype = &population[i];

            let mut removed = false;
            if phenotype.age(generation) > self.max_age {
                removed = true;
                age_count += 1_f32;
            } else if !phenotype.genotype().is_valid() {
                removed = true;
                invalid_count += 1_f32;
            }

            if removed {
                self.replacer
                    .replace(i, generation, population, Arc::clone(&self.encoder));
            }
        }

        let before_species = species.len();
        species.retain(|species| species.age(generation) < self.max_age);
        let species_count = (before_species - species.len()) as f32;

        vec![
            Metric::new_value(metric_names::AGE_FILTER).with_value(age_count),
            Metric::new_value(metric_names::INVALID_FILTER).with_value(invalid_count),
            Metric::new_value(metric_names::SPECIES_FILTER).with_value(species_count),
        ]
    }
}
