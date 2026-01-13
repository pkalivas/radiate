use radiate_core::random_provider;
use radiate_utils::Value;

mod alter;
mod chromosome;
mod codec;
mod fitness;

pub use alter::{PgmParamMutator, PgmScopeMutator};
pub use chromosome::{FactorGene, FactorKind, PgmChromosome, VarSpec};
pub use codec::PgmCodec;
pub use fitness::{PgmDataSet, PgmLogLik};

pub(crate) fn sample_scope(num_vars: usize, max_scope: usize) -> Vec<usize> {
    let k = random_provider::range(1..max_scope.min(num_vars).max(1) + 1);

    let mut picked = Vec::with_capacity(k);
    while picked.len() < k {
        let v = random_provider::range(0..num_vars);
        if !picked.contains(&v) {
            picked.push(v);
        }
    }

    // enforce "child last" convention for Logp when len>1
    if picked.len() > 1 {
        let child_pos = random_provider::range(0..picked.len());
        let child = picked.remove(child_pos);
        picked.push(child);
    }

    picked
}

pub(crate) fn logp_table_shape(vars: &[VarSpec], scope: &[usize]) -> Vec<usize> {
    scope.iter().map(|&vid| vars[vid].card.max(1)).collect()
}

pub(crate) fn init_logp_table(shape: &[usize]) -> Value<f32> {
    Value::from((shape.to_vec(), |_| random_provider::range(-1.0..1.0)))
}
