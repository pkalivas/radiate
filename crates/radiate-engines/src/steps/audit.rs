use crate::steps::EngineStep;
use radiate_core::{
    Chromosome, Ecosystem, Metric, MetricSet, Objective, metric_names, phenotype::PhenotypeId,
};
use radiate_error::Result;
use radiate_utils::intern;
use std::{cmp::Ordering, collections::HashSet};

const EPS: f32 = 1e-9;

#[derive(Default)]
pub struct AuditStep {
    objective: Objective,
    score_distribution: Vec<Vec<f32>>,
    unique_score_work: Vec<Vec<f32>>,
    age_distribution: Vec<usize>,
    seen_ids: HashSet<PhenotypeId>,
    last_gen_ids: HashSet<PhenotypeId>,
}

impl AuditStep {
    pub fn new(objective: Objective) -> Self {
        Self {
            objective,
            ..Default::default()
        }
    }
}

impl AuditStep {
    #[inline]
    fn calc_species_metrics<C: Chromosome>(
        generation: usize,
        metrics: &mut MetricSet,
        ecosystem: &Ecosystem<C>,
    ) {
        if let Some(species) = ecosystem.species() {
            let mut new_species_count = 0;
            let mut species_ages = Vec::with_capacity(species.len());
            let mut species_size = Vec::with_capacity(species.len());

            let pop_len = ecosystem.population().len().max(1);

            let mut max_size = 0;
            let mut size_sum = 0;

            let mut size_vec = Vec::with_capacity(species.len());

            for spec in species.iter() {
                let spec_age = spec.age(generation);

                if spec_age == 0 {
                    new_species_count += 1;
                }

                let len = spec.len();

                species_ages.push(spec_age);
                species_size.push(len);

                max_size = max_size.max(len);
                size_sum += len;
                size_vec.push(len);
            }

            // Largest species share (how dominant is the biggest species)
            let largest_share = if pop_len > 0 {
                max_size as f32 / pop_len as f32
            } else {
                0.0
            };

            let mut largest_share_metric = Metric::new(metric_names::LARGEST_SPECIES_SHARE);
            largest_share_metric.apply_update(largest_share);

            // Species evenness via normalized Shannon entropy
            let mut evenness = 0.0_f32;
            let s_count = species.len();
            if s_count > 1 && size_sum > 0 {
                let size_sum_f = size_sum as f32;
                let mut h = 0.0_f32;
                for sz in size_vec {
                    if sz > 0 {
                        let p = sz as f32 / size_sum_f;
                        h -= p * p.ln();
                    }
                }
                let h_max = (s_count as f32).ln();
                if h_max > 0.0 {
                    evenness = h / h_max;
                }
            }

            // Species churn ratio: new species / total species
            let churn_ratio = if s_count > 0 {
                new_species_count as f32 / s_count as f32
            } else {
                0.0
            };
            let mut churn_metric = Metric::new(metric_names::SPECIES_NEW_RATIO);
            churn_metric.apply_update(churn_ratio);

            metrics.upsert((metric_names::SPECIES_AGE, &species_ages));
            metrics.upsert((metric_names::SPECIES_SIZE, &species_size));
            metrics.upsert((metric_names::SPECIES_COUNT, species.len()));
            metrics.upsert((metric_names::SPECIES_CREATED, new_species_count));
            metrics.upsert((metric_names::SPECIES_EVENNESS, evenness));
            metrics.upsert((metric_names::SPECIES_NEW_RATIO, churn_ratio));
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

        metrics.upsert((metric_names::CARRYOVER_RATE, carryover_rate));
        metrics.upsert((metric_names::NEW_CHILDREN, new_this_gen));
        metrics.upsert((metric_names::SURVIVOR_COUNT, survivor_count));
    }

    #[inline]
    fn calc_derived_metrics<C: Chromosome>(
        _: usize,
        metrics: &mut MetricSet,
        ecosystem: &Ecosystem<C>,
    ) {
        let pop_len = ecosystem.population().len() as f32;
        // Will only compute for single-objective for now
        if let Some(scores) = metrics.get(metric_names::SCORES) {
            let score_coeff = match (scores.value_std_dev(), scores.value_mean()) {
                (Some(std_dev), Some(mean)) if mean != 0.0 => std_dev / mean,
                _ => 0.0,
            };

            metrics.upsert((metric_names::SCORE_VOLATILITY, score_coeff));
        }

        let diversity_ratio = if ecosystem.population().len() > 0 {
            metrics
                .get(metric_names::UNIQUE_MEMBERS)
                .map(|m| m.last_value() / pop_len)
                .unwrap_or(0.0)
        } else {
            0.0
        };

        metrics.upsert((metric_names::DIVERSITY_RATIO, diversity_ratio));
    }

    fn clear_state(&mut self) {
        self.unique_score_work.clear();
        self.age_distribution.clear();
        if self.score_distribution.len() < self.objective.dims() {
            self.score_distribution
                .resize(self.objective.dims(), Vec::new());
        }
        if self.unique_score_work.len() < self.objective.dims() {
            self.unique_score_work
                .resize(self.objective.dims(), Vec::new());
        }

        for vec in &mut self.score_distribution {
            vec.clear();
        }
        for vec in &mut self.unique_score_work {
            vec.clear();
        }
    }
}

impl<C: Chromosome> EngineStep<C> for AuditStep {
    #[inline]
    fn execute(
        &mut self,
        generation: usize,
        ecosystem: &mut Ecosystem<C>,
        metrics: &mut MetricSet,
    ) -> Result<()> {
        self.clear_state();

        let pop = ecosystem.population();
        let n = pop.len();

        self.unique_score_work.reserve_exact(n);
        self.age_distribution.reserve_exact(n);

        let mut size_metric = Vec::with_capacity(n);
        let mut unique_members = HashSet::with_capacity(n);

        for p in pop.iter() {
            unique_members.insert(p.id());

            self.age_distribution.push(p.age(generation));

            let geno_size = p
                .genotype()
                .iter()
                .map(|chromosome| chromosome.len())
                .sum::<usize>();
            size_metric.push(geno_size);

            if let Some(score) = p.score() {
                for (idx, val) in score.iter().enumerate() {
                    self.score_distribution[idx].push(*val);
                    self.unique_score_work[idx].push(*val);
                }
            }
        }

        for vec in &mut self.unique_score_work {
            vec.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
        }

        for (idx, v) in self.unique_score_work.iter().enumerate() {
            let mut unique_count = 0;
            let mut last: Option<f32> = None;
            for val in v {
                if last.map(|l| (l - val).abs() > EPS).unwrap_or(true) {
                    unique_count += 1;
                    last = Some(*val);
                }
            }

            let metric_name = if self.objective.is_single() {
                metric_names::UNIQUE_SCORES
            } else {
                intern!(format!("{}_{}", metric_names::UNIQUE_SCORES, idx))
            };
            metrics.upsert((metric_name, unique_count));
        }

        if !self.score_distribution.is_empty() {
            if self.objective.is_single() {
                metrics.upsert((metric_names::SCORES, &self.score_distribution[0]));
            } else {
                for (idx, vec) in self.score_distribution.iter().enumerate() {
                    let metric_name = intern!(format!("{}_{}", metric_names::SCORES, idx));
                    metrics.upsert((metric_name, vec));
                }
            }
        }

        metrics.upsert((metric_names::AGE, &self.age_distribution));
        metrics.upsert((metric_names::GENOME_SIZE, &size_metric));
        metrics.upsert((metric_names::UNIQUE_MEMBERS, unique_members.len()));

        self.calc_membership_metrics(metrics, ecosystem);
        Self::calc_species_metrics(generation, metrics, ecosystem);
        Self::calc_derived_metrics(generation, metrics, ecosystem);

        Ok(())
    }
}
