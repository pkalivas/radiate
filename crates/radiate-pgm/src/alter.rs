use crate::PgmChromosome;
use radiate_core::alter::{AlterResult, Mutate};
use radiate_core::random_provider;
use radiate_utils::Value;

#[derive(Clone, Debug)]
pub struct PgmParamMutator {
    /// Probability that a given factor is selected for parameter mutation.
    pub factor_rate: f32,
    /// Fraction of entries in a factor table to jitter when selected.
    pub entry_rate: f32,
    /// Jitter magnitude applied to selected logits.
    pub step: f32,
}

impl PgmParamMutator {
    pub fn new(factor_rate: f32, entry_rate: f32, step: f32) -> Self {
        Self {
            factor_rate,
            entry_rate,
            step,
        }
    }
}

impl Mutate<PgmChromosome> for PgmParamMutator {
    #[inline]
    fn mutate_chromosome(&self, chromosome: &mut PgmChromosome, _: f32) -> AlterResult {
        let mut count = 0;
        for f in chromosome.factors.iter_mut() {
            if random_provider::bool(self.factor_rate) {
                let Value::Array { values, .. } = &mut f.params else {
                    continue;
                };

                random_provider::with_rng(|rng| {
                    let n = values.len().max(1);
                    let k = ((n as f32) * self.entry_rate).ceil() as usize;
                    let k = k.clamp(1, n);

                    let idxs = rng.sample_indices(0..n, k);
                    let vals = std::sync::Arc::make_mut(values);

                    for &i in &idxs {
                        vals[i] += rng.range(-self.step..self.step);
                    }

                    k
                });

                count += 1;
            }
        }

        AlterResult::from(count)
    }
}

/// Mutate factor scopes (structure) and rebuild their tables.
#[derive(Clone, Debug)]
pub struct PgmScopeMutator {
    pub factor_rate: f32,
    pub max_scope: usize,
}

impl PgmScopeMutator {
    pub fn new(factor_rate: f32, max_scope: usize) -> Self {
        Self {
            factor_rate,
            max_scope,
        }
    }
}

impl Mutate<PgmChromosome> for PgmScopeMutator {
    #[inline]
    fn mutate_chromosome(&self, chromosome: &mut PgmChromosome, _rate: f32) -> AlterResult {
        let mut count = 0;

        let vars = chromosome.vars.clone();
        for f in chromosome.factors.iter_mut() {
            if random_provider::bool(self.factor_rate) {
                f.resample_scope(&vars, self.max_scope);
                count += 1;
            }
        }

        AlterResult::from(count)
    }
}
