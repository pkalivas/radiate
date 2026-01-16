use crate::{FactorGene, FactorKind, PgmChromosome, factor};
use radiate_core::{BatchFitnessFunction, Genotype, fitness::FitnessFunction};
use radiate_utils::Value;

#[derive(Clone)]
pub struct PgmDataSet {
    pub rows: Vec<Vec<Option<usize>>>,
}

impl PgmDataSet {
    pub fn new(rows: Vec<Vec<Option<usize>>>) -> Self {
        Self { rows }
    }

    pub fn len(&self) -> usize {
        self.rows.len()
    }
}

#[derive(Clone)]
pub struct PgmNll {
    pub data: Vec<Vec<Option<usize>>>,
}

impl FitnessFunction<PgmChromosome, f32> for PgmNll {
    fn evaluate(&self, chrom: PgmChromosome) -> f32 {
        factor::neg_mean_loglik(&chrom, &self.data).unwrap_or(f32::INFINITY) // if something goes off the rails
    }
}

#[derive(Clone)]
pub struct PgmLogLik {
    data: PgmDataSet,
}

impl PgmLogLik {
    pub fn new(data: PgmDataSet) -> Self {
        Self { data }
    }

    fn eval_logp_factor(
        &self,
        chrom: &PgmChromosome,
        factor: &FactorGene,
        row: &[Option<usize>],
    ) -> f32 {
        match factor.kind {
            FactorKind::Logp => {
                let mut idxs = Vec::with_capacity(factor.scope.len());
                for &var_spec in &factor.scope {
                    let Some(s) = row[var_spec.0 as usize] else {
                        return 0.0;
                    };
                    idxs.push(s.min(chrom.vars[var_spec.0 as usize].card.saturating_sub(1)));
                }

                logprob_table_eval(&factor.params, &idxs)
            }
        }
    }

    pub fn loglik(&self, chrom: &PgmChromosome) -> f32 {
        let mut ll = 0.0;
        for row in &self.data.rows {
            for f in &chrom.factors {
                ll += self.eval_logp_factor(chrom, f, row);
            }
        }
        ll
    }

    pub fn neg_mean_loglik(&self, chrom: &PgmChromosome) -> f32 {
        let n = self.data.len().max(1) as f32;
        -(self.loglik(chrom) / n)
    }
}

impl FitnessFunction<PgmChromosome, f32> for PgmLogLik {
    #[inline]
    fn evaluate(&self, input: PgmChromosome) -> f32 {
        self.neg_mean_loglik(&input)
    }
}

impl<'a> FitnessFunction<&'a Genotype<PgmChromosome>, f32> for PgmLogLik {
    #[inline]
    fn evaluate(&self, input: &'a Genotype<PgmChromosome>) -> f32 {
        self.neg_mean_loglik(&input[0])
    }
}

impl BatchFitnessFunction<PgmChromosome, f32> for PgmLogLik {
    #[inline]
    fn evaluate(&self, inputs: Vec<PgmChromosome>) -> Vec<f32> {
        inputs
            .into_iter()
            .map(|c| self.neg_mean_loglik(&c))
            .collect()
    }
}

impl<'a> BatchFitnessFunction<&'a Genotype<PgmChromosome>, f32> for PgmLogLik {
    #[inline]
    fn evaluate(&self, inputs: Vec<&'a Genotype<PgmChromosome>>) -> Vec<f32> {
        inputs
            .into_iter()
            .map(|g| self.neg_mean_loglik(&g[0]))
            .collect()
    }
}

fn logprob_table_eval(val: &Value<f32>, idxs: &[usize]) -> f32 {
    let Value::Array {
        values,
        shape,
        strides,
    } = val
    else {
        return 0.0;
    };

    let rank = shape.rank();
    if idxs.len() != rank {
        return 0.0;
    }
    if rank == 0 {
        return 0.0;
    }

    let child_axis = rank - 1;
    let child = idxs[child_axis];
    let child_states = shape.dim_at(child_axis).max(1);

    // base offset with child fixed to 0
    let mut base = 0usize;
    for i in 0..child_axis {
        let dim = shape.dim_at(i).max(1);
        let idx = idxs[i].min(dim - 1);
        base = base.saturating_add(idx.saturating_mul(strides.stride_at(i)));
    }

    // max logit
    let mut max_logit = f32::NEG_INFINITY;
    for k in 0..child_states {
        let pos = base.saturating_add(k.saturating_mul(strides.stride_at(child_axis)));
        let s = values.get(pos).copied().unwrap_or(0.0);
        max_logit = max_logit.max(s);
    }
    if !max_logit.is_finite() {
        return 0.0;
    }

    // logsumexp
    let mut sum_exp = 0.0f32;
    for k in 0..child_states {
        let pos = base.saturating_add(k.saturating_mul(strides.stride_at(child_axis)));
        let s = values.get(pos).copied().unwrap_or(0.0);
        sum_exp += (s - max_logit).exp();
    }
    let lse = max_logit + sum_exp.ln();

    let child = child.min(child_states.saturating_sub(1));
    let child_pos = base.saturating_add(child.saturating_mul(strides.stride_at(child_axis)));
    let child_logit = values.get(child_pos).copied().unwrap_or(0.0);

    child_logit - lse
}
