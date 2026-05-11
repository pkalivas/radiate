use crate::steps::EngineStep;
use radiate_core::{
    Chromosome, Ecosystem, MetricSet, Objective, SmallStr, metric_names, phenotype::PhenotypeId,
};
use radiate_error::Result;
use std::{cmp::Ordering, collections::HashSet};

const EPS: f32 = 1e-9;

#[derive(Default)]
pub struct AuditStep {
    objective: Objective,
    score_distribution: Vec<Vec<f32>>,
    unique_score_work: Vec<Vec<f32>>,
    age_distribution: Vec<usize>,
    size_distribution: Vec<usize>,
    last_gen_ids: HashSet<PhenotypeId>,
    curr_ids: HashSet<PhenotypeId>,
    unique_members: HashSet<PhenotypeId>,
    scores_per_dim: Vec<SmallStr>,
    unique_scores_per_dim: Vec<SmallStr>,
}

impl AuditStep {
    pub fn new(objective: Objective) -> Self {
        Self {
            objective,
            ..Default::default()
        }
    }

    #[inline]
    fn calc_membership_metrics<C: Chromosome>(
        &mut self,
        metrics: &mut MetricSet,
        ecosystem: &Ecosystem<C>,
    ) {
        self.curr_ids.clear();
        for p in ecosystem.population().iter() {
            self.curr_ids.insert(p.id());
        }

        let pop_len = self.curr_ids.len();

        let survivor_count = self.curr_ids.intersection(&self.last_gen_ids).count();

        let carryover_rate = if pop_len > 0 {
            survivor_count as f32 / pop_len as f32
        } else {
            0.0
        };

        std::mem::swap(&mut self.curr_ids, &mut self.last_gen_ids);

        metrics.upsert(metric_names::CARRYOVER_RATE, carryover_rate);
        metrics.upsert(metric_names::SURVIVOR_COUNT, survivor_count);
    }

    #[inline]
    fn calc_derived_metrics<C: Chromosome>(metrics: &mut MetricSet, ecosystem: &Ecosystem<C>) {
        let pop_len = ecosystem.population().len() as f32;

        // Score volatility: only meaningful when the bare SCORES metric has
        // received any data this run (single-objective path writes it; multi-
        // objective writes per-dim and leaves SCORES empty).
        if let Some(scores) = metrics.get(metric_names::SCORES)
            && scores.count() > 0
        {
            let stddev = scores.stddev();
            let mean = scores.mean();
            let score_coeff = if mean.abs() > EPS {
                stddev / mean.abs()
            } else {
                0.0
            };
            metrics.upsert(metric_names::SCORE_VOLATILITY, score_coeff);
        }

        let diversity_ratio = if !ecosystem.population().is_empty() {
            metrics
                .get(metric_names::UNIQUE_MEMBERS)
                .map(|m| m.last_value() / pop_len)
                .unwrap_or(0.0)
        } else {
            0.0
        };

        metrics.upsert(metric_names::DIVERSITY_RATIO, diversity_ratio);
    }

    fn clear_state(&mut self, pop_size: usize) {
        self.unique_members.clear();

        self.age_distribution.clear();
        self.age_distribution.reserve_exact(pop_size);

        self.size_distribution.clear();
        self.size_distribution.reserve_exact(pop_size);

        let dims = self.objective.dims();
        if self.score_distribution.len() < dims {
            self.score_distribution.resize_with(dims, Vec::new);
        }
        if self.unique_score_work.len() < dims {
            self.unique_score_work.resize_with(dims, Vec::new);
        }

        for v in &mut self.score_distribution {
            v.clear();
            v.reserve_exact(pop_size);
        }
        for v in &mut self.unique_score_work {
            v.clear();
            v.reserve_exact(pop_size);
        }
    }

    fn ensure_per_dim_names(&mut self) {
        if !self.objective.is_multi() {
            return;
        }
        let dims = self.objective.dims();
        if self.scores_per_dim.len() == dims {
            return;
        }
        self.scores_per_dim = (0..dims)
            .map(|i| SmallStr::from_string(format!("{}.{}", metric_names::SCORES, i)))
            .collect();
        self.unique_scores_per_dim = (0..dims)
            .map(|i| SmallStr::from_string(format!("{}.{}", metric_names::UNIQUE_SCORES, i)))
            .collect();
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
        self.ensure_per_dim_names();
        self.clear_state(ecosystem.len());

        let mut new_this_gen = 0;
        for p in ecosystem.population().iter() {
            let age = p.age(generation);
            let geno_size = p
                .genotype()
                .iter()
                .map(|chromosome| chromosome.len())
                .sum::<usize>();

            self.size_distribution.push(geno_size);
            self.unique_members.insert(p.id());
            self.age_distribution.push(age);

            if age == 0 {
                new_this_gen += 1;
            }

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

        let is_single = self.objective.is_single();
        for (idx, v) in self.unique_score_work.iter().enumerate() {
            let mut unique_count = 0;
            let mut last: Option<f32> = None;
            for val in v {
                if last.map(|l| (l - val).abs() > EPS).unwrap_or(true) {
                    unique_count += 1;
                    last = Some(*val);
                }
            }

            if is_single {
                metrics.upsert(metric_names::UNIQUE_SCORES, unique_count);
            } else {
                metrics.upsert(&self.unique_scores_per_dim[idx], unique_count);
            }
        }

        if !self.score_distribution.is_empty() {
            if is_single {
                metrics.upsert(metric_names::SCORES, &self.score_distribution[0]);
            } else {
                for (idx, vec) in self.score_distribution.iter().enumerate() {
                    metrics.upsert(&self.scores_per_dim[idx], vec);
                }
            }
        }

        metrics.upsert(metric_names::NEW_CHILDREN, new_this_gen);
        metrics.upsert(metric_names::AGE, &self.age_distribution);
        metrics.upsert(metric_names::GENOME_SIZE, &self.size_distribution);
        metrics.upsert(metric_names::UNIQUE_MEMBERS, self.unique_members.len());

        self.calc_membership_metrics(metrics, ecosystem);
        Self::calc_derived_metrics(metrics, ecosystem);

        Ok(())
    }
}
