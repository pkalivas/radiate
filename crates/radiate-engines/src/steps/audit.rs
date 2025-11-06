use crate::steps::EngineStep;
use radiate_core::{
    Chromosome, Ecosystem, Metric, MetricSet, metric, metric_names, phenotype::PhenotypeId,
};
use radiate_error::Result;
use std::{cmp::Ordering, collections::HashSet};

const EPS: f32 = 1e-9;

#[derive(Default)]
pub struct AuditStep {
    score_distribution: Vec<f32>,
    unique_score_work: Vec<f32>,
    age_distribution: Vec<usize>,
    seen_ids: HashSet<PhenotypeId>,
    last_gen_ids: HashSet<PhenotypeId>,
}

impl AuditStep {
    #[inline]
    fn calc_species_metrics<C: Chromosome>(
        generation: usize,
        metrics: &mut MetricSet,
        ecosystem: &Ecosystem<C>,
    ) {
        if let Some(species) = ecosystem.species() {
            let mut species_ages = Metric::new(metric_names::SPECIES_AGE);
            let mut new_species_count = Metric::new(metric_names::SPECIES_CREATED);
            let mut species_count = Metric::new(metric_names::SPECIES_COUNT);

            for spec in species.iter() {
                let spec_age = spec.age(generation);

                if spec_age == 0 {
                    new_species_count.apply_update(1);
                }

                species_ages.apply_update(spec_age);
            }

            species_count.apply_update(species.len());

            metrics.add_or_update(new_species_count);
            metrics.add_or_update(species_ages);
            metrics.add_or_update(species_count);
        } else {
            let population_unique_rc_count = ecosystem.population().shared_count();
            assert!(
                population_unique_rc_count == 0,
                "Ecosystem has no species, but population has {} non-unique ptrs",
                population_unique_rc_count
            );
        }
    }

    #[inline]
    fn calc_membership_metrics<C: Chromosome>(
        &mut self,
        metrics: &mut MetricSet,
        ecosystem: &Ecosystem<C>,
    ) {
        let mut curr_ids = HashSet::with_capacity(ecosystem.population().len());
        for p in ecosystem.population().iter() {
            curr_ids.insert(p.id());
        }

        let pop_len = curr_ids.len();

        let new_this_gen = curr_ids.difference(&self.seen_ids).count();
        let survivor_count = curr_ids.intersection(&self.last_gen_ids).count();

        let carryover_rate = if pop_len > 0 {
            survivor_count as f32 / pop_len as f32
        } else {
            0.0
        };

        self.seen_ids.extend(curr_ids.iter().copied());
        drop(std::mem::replace(&mut self.last_gen_ids, curr_ids));

        metrics.add_or_update(metric!(metric_names::NEW_CHILDREN, new_this_gen));
        metrics.add_or_update(metric!(metric_names::SURVIVOR_COUNT, survivor_count));
        metrics.add_or_update(metric!(metric_names::CARRYOVER_RATE, carryover_rate));
        metrics.add_or_update(metric!(
            metric_names::LIFETIME_UNIQUE_MEMBERS,
            self.seen_ids.len()
        ));
    }

    #[inline]
    fn calc_derived_metrics<C: Chromosome>(
        _: usize,
        metrics: &mut MetricSet,
        ecosystem: &Ecosystem<C>,
    ) {
        let pop_len = ecosystem.population().len() as f32;
        let derived_scores = metrics.get(metric_names::SCORES).map(|score| {
            let score_coeff = match (score.distribution_std_dev(), score.distribution_mean()) {
                (Some(std_dev), Some(mean)) if mean != 0.0 => std_dev / mean,
                _ => 0.0,
            };

            let diversity_ratio = if ecosystem.population().len() > 0 {
                metrics
                    .get(metric_names::UNIQUE_MEMBERS)
                    .map(|m| m.last_value() / pop_len)
                    .unwrap_or(0.0)
            } else {
                0.0
            };

            let coeff_metric = metric!(metric_names::SCORE_VOLATILITY, score_coeff);
            let diversity_metric = metric!(metric_names::DIVERSITY_RATIO, diversity_ratio);

            [coeff_metric, diversity_metric]
        });

        if let Some([coeff_metric, diversity_metric]) = derived_scores {
            metrics.add_or_update(coeff_metric);
            metrics.add_or_update(diversity_metric);
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
    ) -> Result<()> {
        let pop = ecosystem.population();
        let n = pop.len();

        self.score_distribution.clear();
        self.unique_score_work.clear();
        self.age_distribution.clear();
        self.score_distribution.reserve_exact(n);
        self.unique_score_work.reserve_exact(n);
        self.age_distribution.reserve_exact(n);

        let mut age_metric = Metric::new(metric_names::AGE);
        let mut size_metric = Metric::new(metric_names::GENOME_SIZE);
        let mut unique_members = HashSet::with_capacity(n);

        for p in pop.iter() {
            unique_members.insert(p.id());

            age_metric.apply_update(p.age(generation));

            let geno_size = p
                .genotype()
                .iter()
                .map(|chromosome| chromosome.len())
                .sum::<usize>();
            size_metric.apply_update(geno_size);

            if let Some(s) = p.score() {
                let v = s.as_f32();
                self.score_distribution.push(v);
                self.unique_score_work.push(v);
            }
        }

        let mut equal_metric = Metric::new(metric_names::UNIQUE_MEMBERS);
        equal_metric.apply_update(unique_members.len());

        self.unique_score_work
            .sort_unstable_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
        let mut unique_count = 0;
        let mut last: Option<f32> = None;
        for v in &self.unique_score_work {
            if last.map(|l| (l - v).abs() > EPS).unwrap_or(true) {
                unique_count += 1;
                last = Some(*v);
            }
        }

        let mut unique_metric = Metric::new(metric_names::UNIQUE_SCORES);
        unique_metric.apply_update(unique_count);

        let mut score_metric = Metric::new(metric_names::SCORES);
        if !self.score_distribution.is_empty() {
            score_metric.apply_update(&self.score_distribution);
        }

        metrics.add_or_update(age_metric);
        metrics.add_or_update(size_metric);
        metrics.add_or_update(equal_metric);
        metrics.add_or_update(unique_metric);
        metrics.add_or_update(score_metric);

        Self::calc_species_metrics(generation, metrics, ecosystem);
        self.calc_membership_metrics(metrics, ecosystem);
        Self::calc_derived_metrics(generation, metrics, ecosystem);

        Ok(())
    }
}
