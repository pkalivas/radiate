use super::{Chromosome, Metric, Population, metric_names};
use std::{collections::HashSet, vec};

pub trait Audit<C: Chromosome> {
    fn audit(&self, generation: usize, population: &Population<C>) -> Vec<Metric>;
}

impl<C: Chromosome, F: Fn(usize, &Population<C>) -> Vec<Metric>> Audit<C> for F {
    fn audit(&self, generation: usize, population: &Population<C>) -> Vec<Metric> {
        self(generation, population)
    }
}

/// Adds various metrics to the output context, including the age of individuals, the score of individuals,
/// and the number of unique scores in the population. These metrics can be used to monitor the progress of
/// the genetic algorithm and to identify potential issues or areas for improvement.
///
/// The age of an individual is the number of generations it has survived, while the score of an individual
/// is a measure of its fitness. The number of unique scores in the population is a measure of diversity, with
/// a higher number indicating a more diverse population.
pub struct MetricAudit;

impl<C: Chromosome> Audit<C> for MetricAudit {
    fn audit(&self, generation: usize, population: &Population<C>) -> Vec<Metric> {
        let mut age_metric = Metric::new_value(metric_names::AGE);
        let mut score_metric = Metric::new_value(metric_names::SCORE);
        let mut size_metric = Metric::new_distribution(metric_names::GENOME_SIZE);
        let mut unique_scores = Vec::with_capacity(population.len());
        let mut unique_members = HashSet::new();

        for phenotype in population.iter() {
            unique_members.insert(phenotype.id());

            let age = phenotype.age(generation);
            let score = phenotype.score();
            let phenotype_size = phenotype
                .genotype()
                .iter()
                .map(|chromosome| chromosome.len())
                .sum::<usize>();

            age_metric.add_value(age as f32);
            score_metric.add_value(score.map(|sc| sc.as_f32()).unwrap_or_default());
            unique_scores.push(score);
            size_metric.add_value(phenotype_size as f32);
        }

        unique_scores.dedup();

        let mut unique_metric = Metric::new_value(metric_names::UNIQUE_SCORES);
        let mut equal_metric = Metric::new_value(metric_names::UNIQUE_MEMBERS);

        unique_metric.add_value(unique_scores.len() as f32);
        equal_metric.add_value(unique_members.len() as f32);

        vec![
            age_metric,
            score_metric,
            unique_metric,
            size_metric,
            equal_metric,
        ]
    }
}
