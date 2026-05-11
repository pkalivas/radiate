use crate::steps::EngineStep;
use radiate_core::{
    Chromosome, Ecosystem, MetricSet, Objective, define_metric_handles, metric_names,
    phenotype::PhenotypeId,
};
use radiate_error::Result;
use std::{cmp::Ordering, collections::HashSet};

const EPS: f32 = 1e-9;

define_metric_handles! {
    pub struct AuditHandles {
        // Always-on scalars
        carryover_rate    = metric_names::CARRYOVER_RATE,
        new_children      = metric_names::NEW_CHILDREN,
        survivor_count    = metric_names::SURVIVOR_COUNT,
        diversity_ratio   = metric_names::DIVERSITY_RATIO,
        score_volatility  = metric_names::SCORE_VOLATILITY,
        age               = metric_names::AGE,
        genome_size       = metric_names::GENOME_SIZE,
        unique_members    = metric_names::UNIQUE_MEMBERS,

        // Single-objective scalars (idle in multi-objective)
        scores            = metric_names::SCORES,
        unique_scores     = metric_names::UNIQUE_SCORES,
    }
}

define_metric_handles! {
    pub struct SpeciesHandles {
        species_age           = metric_names::SPECIES_AGE,
        species_size          = metric_names::SPECIES_SIZE,
        species_count         = metric_names::SPECIES_COUNT,
        species_created       = metric_names::SPECIES_CREATED,
        species_evenness      = metric_names::SPECIES_EVENNESS,
        species_new_ratio     = metric_names::SPECIES_NEW_RATIO,
        largest_species_share = metric_names::LARGEST_SPECIES_SHARE,
    }
}

define_metric_handles! {
    pub struct MultiObjectiveHandles {
        scores_per_dim[]        = metric_names::SCORES,
        unique_scores_per_dim[] = metric_names::UNIQUE_SCORES,
    }
}

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
    handles: AuditHandles,
    species_handles: SpeciesHandles,
    multiobj_handles: MultiObjectiveHandles,
}

impl AuditStep {
    pub fn new(objective: Objective) -> Self {
        Self {
            objective,
            ..Default::default()
        }
    }

    #[inline]
    fn calc_species_metrics<C: Chromosome>(
        handles: &SpeciesHandles,
        generation: usize,
        metrics: &mut MetricSet,
        ecosystem: &Ecosystem<C>,
    ) {
        let Some(species) = ecosystem.species() else {
            return;
        };

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

        metrics.upsert_at(handles.species_age, &species_ages);
        metrics.upsert_at(handles.species_size, &species_size);
        metrics.upsert_at(handles.species_count, species.len());
        metrics.upsert_at(handles.species_created, new_species_count);
        metrics.upsert_at(handles.species_evenness, evenness);
        metrics.upsert_at(handles.species_new_ratio, churn_ratio);
        metrics.upsert_at(handles.largest_species_share, largest_share);
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

        metrics.upsert_at(self.handles.carryover_rate, carryover_rate);
        metrics.upsert_at(self.handles.survivor_count, survivor_count);
    }

    #[inline]
    fn calc_derived_metrics<C: Chromosome>(
        handles: &AuditHandles,
        metrics: &mut MetricSet,
        ecosystem: &Ecosystem<C>,
    ) {
        let pop_len = ecosystem.population().len() as f32;

        // Score volatility: only meaningful when the bare SCORES metric has
        // received any data this run (single-objective path writes it; multi-
        // objective writes per-dim and leaves SCORES empty).
        if let Some(scores) = metrics.get_by_idx(handles.scores)
            && scores.count() > 0
        {
            let stddev = scores.stddev();
            let mean = scores.mean();
            let score_coeff = if mean.abs() > EPS {
                stddev / mean.abs()
            } else {
                0.0
            };
            metrics.upsert_at(handles.score_volatility, score_coeff);
        }

        let diversity_ratio = if !ecosystem.population().is_empty() {
            metrics
                .get_by_idx(handles.unique_members)
                .map(|m| m.last_value() / pop_len)
                .unwrap_or(0.0)
        } else {
            0.0
        };

        metrics.upsert_at(handles.diversity_ratio, diversity_ratio);
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
}

impl<C: Chromosome> EngineStep<C> for AuditStep {
    #[inline]
    fn execute(
        &mut self,
        generation: usize,
        ecosystem: &mut Ecosystem<C>,
        metrics: &mut MetricSet,
    ) -> Result<()> {
        let dims = self.objective.dims();
        self.handles.ensure(metrics, dims);

        if ecosystem.species().is_some() {
            self.species_handles.ensure(metrics, dims);
        }

        if self.objective.is_multi() {
            self.multiobj_handles.ensure(metrics, dims);
        }

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

            let handle = if is_single {
                self.handles.unique_scores
            } else {
                self.multiobj_handles.unique_scores_per_dim[idx]
            };

            metrics.upsert_at(handle, unique_count);
        }

        if !self.score_distribution.is_empty() {
            if is_single {
                metrics.upsert_at(self.handles.scores, &self.score_distribution[0]);
            } else {
                for (idx, vec) in self.score_distribution.iter().enumerate() {
                    metrics.upsert_at(self.multiobj_handles.scores_per_dim[idx], vec);
                }
            }
        }

        metrics.upsert_at(self.handles.new_children, new_this_gen);
        metrics.upsert_at(self.handles.age, &self.age_distribution);
        metrics.upsert_at(self.handles.genome_size, &self.size_distribution);
        metrics.upsert_at(self.handles.unique_members, self.unique_members.len());

        self.calc_membership_metrics(metrics, ecosystem);
        Self::calc_species_metrics(&self.species_handles, generation, metrics, ecosystem);
        Self::calc_derived_metrics(&self.handles, metrics, ecosystem);

        Ok(())
    }
}

// fn topk_share(mut counts: Vec<usize>, k: usize) -> f32 {
//     if counts.is_empty() {
//         return 0.0;
//     }
//     counts.sort_unstable_by(|a, b| b.cmp(a));
//     let total: usize = counts.iter().sum();
//     if total == 0 {
//         return 0.0;
//     }
//     let take = counts.into_iter().take(k).sum::<usize>();
//     take as f32 / total as f32
// }

// fn normalized_shannon_entropy(counts: &[usize]) -> f32 {
//     println!("counts: {:?}", counts);
//     let total: usize = counts.iter().sum();
//     if total == 0 {
//         return 0.0;
//     }

//     let total_f = total as f32;
//     let mut h = 0.0f32;
//     let mut k = 0usize;

//     for &c in counts {
//         if c == 0 {
//             continue;
//         }
//         k += 1;
//         let p = c as f32 / total_f;
//         h -= p * p.ln();
//     }

//     if k <= 1 {
//         return 0.0;
//     }
//     let h_max = (k as f32).ln();
//     if h_max <= 0.0 { 0.0 } else { h / h_max }
// }
