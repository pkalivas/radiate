use crate::steps::EngineStep;
use radiate_core::{Chromosome, Ecosystem, Metric, MetricSet, metric, metric_names};
use std::collections::HashSet;

pub struct AuditStep;

impl AuditStep {
    fn calc_metrics<C: Chromosome>(
        generation: usize,
        metrics: &mut MetricSet,
        ecosystem: &mut Ecosystem<C>,
    ) {
        let mut age_metric = Metric::new(metric_names::AGE);
        let mut score_metric = Metric::new(metric_names::SCORES);
        let mut size_metric = Metric::new(metric_names::GENOME_SIZE);
        let mut unique_metric = Metric::new(metric_names::UNIQUE_SCORES);
        let mut equal_metric = Metric::new(metric_names::UNIQUE_MEMBERS);

        let mut unique_members = HashSet::new();
        let mut unique_scores = Vec::with_capacity(ecosystem.population().len());

        let mut score_distribution = Vec::with_capacity(ecosystem.population().len());
        for phenotype in ecosystem.population().iter() {
            unique_members.insert(phenotype.id());

            let age = phenotype.age(generation);
            let score = phenotype.score();
            let phenotype_size = phenotype
                .genotype()
                .iter()
                .map(|chromosome| chromosome.len())
                .sum::<usize>();

            age_metric.apply_update(age);
            score_metric.apply_update(score.map(|sc| sc.as_f32()).unwrap_or_default());
            score_distribution.push(phenotype.score().unwrap().as_f32());

            unique_scores.push(score);
            size_metric.apply_update(phenotype_size);
        }

        unique_scores.dedup();

        score_metric.apply_update(&score_distribution);
        unique_metric.apply_update(unique_scores.len());
        equal_metric.apply_update(unique_members.len());

        metrics.add_or_update(age_metric);
        metrics.add_or_update(score_metric);
        metrics.add_or_update(unique_metric);
        metrics.add_or_update(size_metric);
        metrics.add_or_update(equal_metric);
    }

    fn calc_species_metrics<C: Chromosome>(
        generation: usize,
        metrics: &mut MetricSet,
        ecosystem: &mut Ecosystem<C>,
    ) {
        if let Some(species) = ecosystem.species() {
            let mut species_ages = Metric::new(metric_names::SPECIES_AGE);
            let mut new_species_count = Metric::new(metric_names::SPECIES_CREATED);
            let mut species_count = Metric::new(metric_names::SPECIES_COUNT);

            for spec in species.iter() {
                let spec_age = spec.age(generation);

                if spec_age > 0 {
                    new_species_count.apply_update(1);
                }

                species_ages.apply_update(spec_age);
            }

            species_count.apply_update(species.len());

            metrics.add_or_update(new_species_count);
            metrics.add_or_update(species_ages);
            metrics.add_or_update(species_count);
        }
    }

    fn calc_derived_metrics<C: Chromosome>(
        _: usize,
        metrics: &mut MetricSet,
        ecosystem: &mut Ecosystem<C>,
    ) {
        let derived_scores = metrics.get(metric_names::SCORES).map(|score| {
            let score_coeff = match (score.value_std_dev(), score.value_mean()) {
                (Some(std_dev), Some(mean)) if mean != 0.0 => std_dev / mean,
                _ => 0.0,
            };

            let diversity_ratio = if ecosystem.population().len() > 0 {
                metrics
                    .get(metric_names::UNIQUE_MEMBERS)
                    .map(|m| m.last_value() / ecosystem.population().len() as f32)
                    .unwrap_or(0.0)
            } else {
                0.0
            };

            let coeff_metric = metric!(metric_names::SCORE_VOLATILITY, score_coeff);
            let diversity_metric = metric!(metric_names::DIVERSITY_RATIO, diversity_ratio);

            vec![coeff_metric, diversity_metric]
        });

        if let Some(derived_scores) = derived_scores {
            for metric in derived_scores {
                metrics.add_or_update(metric);
            }
        }
    }
}

impl<C: Chromosome> EngineStep<C> for AuditStep {
    #[inline]
    fn execute(
        &mut self,
        generation: usize,
        metrics: &mut MetricSet,
        ecosystem: &mut Ecosystem<C>,
    ) {
        Self::calc_metrics(generation, metrics, ecosystem);
        Self::calc_species_metrics(generation, metrics, ecosystem);
        Self::calc_derived_metrics(generation, metrics, ecosystem);
    }
}
