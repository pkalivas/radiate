use crate::steps::EngineStep;
use radiate_core::{
    Chromosome, Ecosystem, Evaluate, MetricQuery, MetricSet, MetricUpdate, Objective, Score,
    SmallStr, math::distribution, metric_names, phenotype::PhenotypeId, stats::TagType,
};
use radiate_error::Result;
use std::{
    cmp::Ordering,
    collections::HashSet,
    sync::{Arc, Mutex},
};

const EPS: f32 = 1e-9;

#[derive(Default)]
pub struct MetricStep {
    objective: Objective,
    best_score: Option<Score>,

    expressions: Option<Arc<Mutex<Vec<MetricQuery>>>>,

    score_dist_per_dim: Vec<Vec<f32>>,
    unique_scores_per_dim: Vec<Vec<f32>>,

    score_sum_per_dim: Vec<f64>,
    score_sum_squared_per_dim: Vec<f64>,
    sum_score_size_per_dim: Vec<f64>,

    age_distribution: Vec<usize>,
    size_distribution: Vec<usize>,

    last_gen_ids: HashSet<PhenotypeId>,
    curr_ids: HashSet<PhenotypeId>,
    unique_members: HashSet<PhenotypeId>,

    score_names: Vec<SmallStr>,
    unique_score_names: Vec<SmallStr>,
    evenness_names: Vec<SmallStr>,
    gini_names: Vec<SmallStr>,
    corr_names: Vec<SmallStr>,
    best_score_names: Vec<SmallStr>,
}

impl MetricStep {
    pub fn new(objective: Objective, expressions: Option<Arc<Mutex<Vec<MetricQuery>>>>) -> Self {
        Self {
            objective,
            expressions,
            ..Default::default()
        }
    }

    #[inline]
    fn calc_membership_metrics<C: Chromosome>(
        &mut self,
        metrics: &mut MetricSet,
        _: &Ecosystem<C>,
    ) {
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

    #[inline]
    fn calc_improvement_metrics<C: Chromosome>(
        &mut self,
        metrics: &mut MetricSet,
        ecosystem: &Ecosystem<C>,
    ) {
        let mut best_improved = false;
        let current_best = ecosystem.get_phenotype(0).and_then(|p| p.score());

        if self.best_score.is_none() {
            self.best_score = current_best.cloned();
            best_improved = true;
        } else if let (Some(current), Some(best)) = (current_best, &self.best_score)
            && self.objective.is_better(current, best)
        {
            self.best_score = Some(current.clone());
            best_improved = true;
        }

        metrics.upsert(metric_names::BEST_SCORE_IMPROVEMENT, best_improved);

        if let Some(score) = &self.best_score {
            if score.len() == 1 {
                metrics.upsert(metric_names::BEST_SCORES, score[0]);
            } else {
                for (i, score) in score.iter().enumerate() {
                    let name = &self.best_score_names[i];
                    metrics.upsert(name, *score);
                }
            }
        }
    }

    #[inline]
    fn calc_expression_metrics(&mut self, metrics: &mut MetricSet) -> Result<()> {
        if let Some(exprs) = &self.expressions {
            let mut exprs = exprs.lock().unwrap();
            for expr in exprs.iter_mut() {
                let (name, exp) = expr.pair();
                let update = MetricUpdate::try_from(exp.eval(metrics)?)?;

                metrics.upsert_tagged(name, update, TagType::Expr);
            }
        }

        Ok(())
    }

    fn clear_state(&mut self) {
        self.unique_members.clear();
        self.age_distribution.clear();
        self.size_distribution.clear();
        self.curr_ids.clear();
        self.score_sum_per_dim.clear();
        self.score_sum_squared_per_dim.clear();
        self.sum_score_size_per_dim.clear();

        let dims = self.objective.dims();
        if self.score_dist_per_dim.len() < dims {
            self.score_dist_per_dim.resize_with(dims, Vec::new);
        }
        if self.unique_scores_per_dim.len() < dims {
            self.unique_scores_per_dim.resize_with(dims, Vec::new);
        }
        if self.score_sum_per_dim.len() < dims {
            self.score_sum_per_dim.resize(dims, 0.0);
        }
        if self.score_sum_squared_per_dim.len() < dims {
            self.score_sum_squared_per_dim.resize(dims, 0.0);
        }
        if self.sum_score_size_per_dim.len() < dims {
            self.sum_score_size_per_dim.resize(dims, 0.0);
        }

        for v in &mut self.score_dist_per_dim {
            v.clear();
        }
        for v in &mut self.unique_scores_per_dim {
            v.clear();
        }
    }

    fn ensure_per_dim_names(&mut self) {
        if !self.objective.is_multi() {
            return;
        }

        let dims = self.objective.dims();
        if self.score_names.len() == dims {
            return;
        }

        self.score_names = per_dim_names(&metric_names::SCORES, dims);
        self.unique_score_names = per_dim_names(&metric_names::UNIQUE_SCORES, dims);
        self.evenness_names = per_dim_names(&metric_names::SCORES_EVENNESS, dims);
        self.gini_names = per_dim_names(&metric_names::SCORES_GINI, dims);
        self.corr_names = per_dim_names(&metric_names::SIZE_SCORE_CORR, dims);
        self.best_score_names = per_dim_names(&metric_names::BEST_SCORES, dims);
    }
}

impl<C: Chromosome> EngineStep<C> for MetricStep {
    #[inline]
    fn execute(
        &mut self,
        generation: usize,
        ecosystem: &mut Ecosystem<C>,
        metrics: &mut MetricSet,
    ) -> Result<()> {
        self.ensure_per_dim_names();
        self.clear_state();

        // Streaming accumulators for the genome-size ↔ fitness correlation
        // (bloat signal). Only the scored members contribute, so size sums are
        // gathered here rather than from `size_distribution` (which spans every
        // member, scored or not). f64 to stay stable across large populations.
        let dims = self.objective.dims();
        let mut sum_size = 0.0f64;
        let mut sum_size2 = 0.0f64;

        let mut new_this_gen = 0;
        for p in ecosystem.population().iter() {
            let Some(score) = p.score() else {
                continue;
            };

            if !self.objective.validate(score) {
                return Err(radiate_error::RadiateError::Fitness(format!(
                    "Score {:?} has invalid dimensions for the objective {:?}.",
                    score, self.objective
                )));
            }

            let id = p.id();
            let age = p.age(generation);
            let geno_size = p
                .genotype()
                .iter()
                .map(|chromosome| chromosome.len())
                .sum::<usize>();

            self.size_distribution.push(geno_size);
            self.unique_members.insert(id);
            self.age_distribution.push(age);
            self.curr_ids.insert(id);

            if age == 0 {
                new_this_gen += 1;
            }

            let sz = geno_size as f64;
            sum_size += sz;
            sum_size2 += sz * sz;

            for (idx, val) in score.iter().enumerate() {
                self.score_dist_per_dim[idx].push(*val);
                self.unique_scores_per_dim[idx].push(*val);

                let v = *val as f64;
                self.score_sum_per_dim[idx] += v;
                self.score_sum_squared_per_dim[idx] += v * v;
                self.sum_score_size_per_dim[idx] += sz * v;
            }
        }

        for vec in &mut self.unique_scores_per_dim {
            vec.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
        }

        let is_single = self.objective.is_single();
        for (idx, v) in self.unique_scores_per_dim.iter().enumerate() {
            let shape = distribution::shape(v);

            if is_single {
                metrics.upsert(metric_names::UNIQUE_SCORES, shape.unique);
                metrics.upsert(metric_names::SCORES_EVENNESS, shape.evenness);
                metrics.upsert(metric_names::SCORES_GINI, shape.gini);
            } else {
                metrics.upsert(&self.unique_score_names[idx], shape.unique);
                metrics.upsert(&self.evenness_names[idx], shape.evenness);
                metrics.upsert(&self.gini_names[idx], shape.gini);
            }
        }

        if !self.score_dist_per_dim.is_empty() {
            if is_single {
                metrics.upsert(metric_names::SCORES, &self.score_dist_per_dim[0]);
            } else {
                for (idx, vec) in self.score_dist_per_dim.iter().enumerate() {
                    metrics.upsert(&self.score_names[idx], vec);
                }
            }
        }

        // Genome-size ↔ fitness correlation (Pearson r per objective). Only
        // defined when genome length actually varies across the population —
        // i.e. variable-length GP genomes; for fixed-length genomes the size
        // variance is zero and the metric is omitted rather than reported as 0.
        let n = ecosystem.len() as f64;
        let var_size = sum_size2 - sum_size * sum_size / n;
        if var_size > EPS as f64 {
            let denom_size = var_size.sqrt();
            for idx in 0..dims {
                let sum_score = self.score_sum_per_dim[idx];
                let square_sum_score = self.score_sum_squared_per_dim[idx];
                let sum_size_score = self.sum_score_size_per_dim[idx];

                let var_score = square_sum_score - sum_score * sum_score / n;
                if var_score <= EPS as f64 {
                    continue;
                }

                let cov = sum_size_score - sum_size * sum_score / n;
                let r = (cov / (denom_size * var_score.sqrt())) as f32;

                if is_single {
                    metrics.upsert(metric_names::SIZE_SCORE_CORR, r);
                } else {
                    metrics.upsert(&self.corr_names[idx], r);
                }
            }
        }

        metrics.upsert(metric_names::NEW_CHILDREN, new_this_gen);
        metrics.upsert(metric_names::AGE, &self.age_distribution);
        metrics.upsert(metric_names::GENOME_SIZE, &self.size_distribution);
        metrics.upsert(metric_names::UNIQUE_MEMBERS, self.unique_members.len());

        self.calc_membership_metrics(metrics, ecosystem);
        Self::calc_derived_metrics(metrics, ecosystem);
        self.calc_improvement_metrics(metrics, ecosystem);
        self.calc_expression_metrics(metrics)?;

        Ok(())
    }
}

fn per_dim_names(base: &SmallStr, dims: usize) -> Vec<SmallStr> {
    (0..dims)
        .map(|i| SmallStr::from_string(format!("{}.{}", base, i)))
        .collect()
}
