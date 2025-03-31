use super::EngineStep;
use crate::{
    Chromosome, GeneticEngineParams, Metric, Objective, Population, Scored, Species, metric_names,
};
use std::collections::HashSet;

pub struct AuditStep {
    objective: Objective,
}

/// Audits the current state of the genetic algorithm, updating the best individual found so far
/// and calculating various metrics such as the age of individuals, the score of individuals, and the
/// number of unique scores in the population. This method is called at the end of each generation.
impl<C, T> EngineStep<C, T> for AuditStep
where
    C: Chromosome + 'static,
    T: Clone + Send + 'static,
{
    fn register(params: &GeneticEngineParams<C, T>) -> Option<Box<Self>>
    where
        Self: Sized,
    {
        Some(Box::new(AuditStep {
            objective: params.objective().clone(),
        }))
    }

    fn execute(
        &self,
        generation: usize,
        population: &mut Population<C>,
        species: &mut Vec<Species<C>>,
    ) -> Vec<Metric> {
        self.objective.sort(population);

        let mut age_metric = Metric::new_value(metric_names::AGE);
        let mut score_metric = Metric::new_value(metric_names::SCORE);
        let mut size_values = Vec::with_capacity(population.len());
        let mut unique = Vec::with_capacity(population.len());
        let mut equal_members = HashSet::new();
        let mut new_species_count = 0;
        let mut species_ages = Metric::new_value(metric_names::SPECIES_AGE);

        for species in species.iter() {
            let species_age = species.age(generation);

            if species_age > 0 {
                new_species_count += 1;
            }

            species_ages.add_value(species_age as f32);
        }

        for i in 0..population.len() {
            let phenotype = &population[i];

            equal_members.insert(phenotype.id());

            let age = phenotype.age(generation);
            let score = phenotype.score();
            let phenotype_size = phenotype
                .genotype()
                .iter()
                .map(|chromosome| chromosome.len())
                .sum::<usize>();

            age_metric.add_value(age as f32);
            score_metric.add_value(score.as_f32());
            unique.push(score.clone());
            size_values.push(phenotype_size as f32);
        }

        unique.dedup();

        let mut unique_scores = Metric::new_value(metric_names::UNIQUE_SCORES);
        let mut size_metric = Metric::new_distribution(metric_names::GENOME_SIZE);
        let mut unique_individuals = Metric::new_value(metric_names::UNIQUE_INDIVIDUALS);

        unique_scores.add_value(unique.len() as f32);
        size_metric.add_sequence(&size_values);
        unique_individuals.add_value(equal_members.len() as f32);

        let mut result = vec![
            age_metric,
            score_metric,
            unique_scores,
            size_metric,
            unique_individuals,
        ];

        if new_species_count > 0 {
            result.push(
                Metric::new_value(metric_names::SPECIES_CREATED)
                    .with_value(new_species_count as f32),
            );
        }

        if species_ages.count() > 0 {
            result.push(species_ages);
        }

        if species.len() > 0 {
            result.push(
                Metric::new_value(metric_names::SPECIES_COUNT).with_value(species.len() as f32),
            );
        }

        result
    }
}

// impl<C: Chromosome> Audit<C> for MetricAudit {
//     fn audit(&self, generation: usize, population: &Population<C>) -> Vec<Metric> {
//         let mut age_metric = Metric::new_value(metric_names::AGE);
//         let mut score_metric = Metric::new_value(metric_names::SCORE);
//         let mut size_values = Vec::with_capacity(population.len());
//         let mut unique = Vec::with_capacity(population.len());
//         let mut equal_members = HashSet::new();

//         for i in 0..population.len() {
//             let phenotype = &population[i];

//             equal_members.insert(phenotype.id());

//             let age = phenotype.age(generation);
//             let score = phenotype.score();
//             let phenotype_size = phenotype
//                 .genotype()
//                 .iter()
//                 .map(|chromosome| chromosome.len())
//                 .sum::<usize>();

//             age_metric.add_value(age as f32);
//             score_metric.add_value(score.as_f32());
//             unique.push(score.clone());
//             size_values.push(phenotype_size as f32);
//         }

//         unique.dedup();

//         let mut unique_scores = Metric::new_value(metric_names::UNIQUE_SCORES);
//         let mut size_metric = Metric::new_distribution(metric_names::GENOME_SIZE);
//         let mut unique_individuals = Metric::new_value(metric_names::UNIQUE_INDIVIDUALS);

//         unique_scores.add_value(unique.len() as f32);
//         size_metric.add_sequence(&size_values);
//         unique_individuals.add_value(equal_members.len() as f32);

//         vec![
//             age_metric,
//             score_metric,
//             unique_scores,
//             size_metric,
//             unique_individuals,
//         ]
//     }
// }
