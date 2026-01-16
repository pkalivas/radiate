mod discrete;

pub use discrete::DiscreteFactor;
use radiate_utils::Value;

use crate::{FactorGene, PgmChromosome, VarId, VarSpec, variable_elimination};

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub enum Factor {
    Discrete(DiscreteFactor),
    // Later: Gaussian, LinearGaussian, CLG, etc.
}

impl Factor {
    #[allow(dead_code)]
    pub fn scope(&self) -> &[crate::var::VarId] {
        match self {
            Factor::Discrete(f) => f.scope(),
        }
    }
}

/// Convert a single `FactorGene` into a `DiscreteFactor` using the chromosome's `vars` table.
pub fn gene_to_discrete(chrom: &PgmChromosome, g: &FactorGene) -> Result<DiscreteFactor, String> {
    // Build VarSpec scope (order preserved)
    let scope_specs: Vec<VarSpec> = g
        .scope
        .iter()
        .map(|&vid| chrom.vars[vid.0 as usize].clone())
        .collect();

    // Extract contiguous values from `Value`.
    let Value::Array { values, .. } = &g.params else {
        return Err("FactorGene.params must be a Value::Array".into());
    };

    DiscreteFactor::new(scope_specs, values.to_vec())
}

/// Convert a chromosome into math factors.
pub fn chromosome_factors(chrom: &PgmChromosome) -> Result<Vec<DiscreteFactor>, String> {
    chrom
        .factors
        .iter()
        .map(|g| gene_to_discrete(chrom, g))
        .collect()
}

/// Multiply all chromosome factors into a single joint factor.
/// This is exact but can be huge; intended for small models / debugging.
pub fn joint_factor(chrom: &PgmChromosome) -> Result<DiscreteFactor, String> {
    let factors = chromosome_factors(chrom)?;
    if factors.is_empty() {
        return Err("no factors".into());
    }

    let card = |v: VarId| chrom.vars[v.0 as usize].card;

    let mut joint = factors[0].clone();
    for f in factors.iter().skip(1) {
        joint = joint.product(f, &card)?;
    }
    Ok(joint)
}

/// Compute a marginal over `keep` variables by building the joint and marginalizing out others.
pub fn marginal_joint(chrom: &PgmChromosome, keep: &[VarId]) -> Result<DiscreteFactor, String> {
    let joint = joint_factor(chrom)?;
    let elim: Vec<VarId> = joint
        .scope()
        .iter()
        .copied()
        .filter(|v| !keep.contains(v))
        .collect();
    joint.marginalize(&elim)
}

/// Compute a marginal using variable elimination (preferred).
/// `elim_order` is used as-is; if `None`, it defaults to eliminating all vars not in `keep` in id order.
pub fn marginal_ve(
    chrom: &PgmChromosome,
    keep: &[VarId],
    elim_order: Option<&[VarId]>,
) -> Result<DiscreteFactor, String> {
    let factors = chromosome_factors(chrom)?;
    let card = |v: VarId| chrom.vars[v.0 as usize].card;

    let order: Vec<VarId> = match elim_order {
        Some(o) => o.to_vec(),
        None => {
            let mut all: Vec<VarId> = (0..chrom.vars.len())
                .map(|i| VarId(i as u32))
                .filter(|v| !keep.contains(v))
                .collect();
            all.sort_by_key(|v| v.0);
            all
        }
    };

    let out = variable_elimination(factors, &order, &card)?;

    // If VE leaves more than keep vars (e.g. disconnected components), marginalize down.
    let extra: Vec<VarId> = out
        .scope()
        .iter()
        .copied()
        .filter(|v| !keep.contains(v))
        .collect();

    if extra.is_empty() {
        Ok(out)
    } else {
        out.marginalize(&extra)
    }
}

/// Evidence log-likelihood for a row with missing values.
/// This returns log( sum_{hidden} exp( sum_f logphi_f(x) ) ).
/// For a normalized model, subtract logZ to get log P(evidence).
pub fn loglik_evidence(chrom: &PgmChromosome, row: &[Option<usize>]) -> Result<f32, String> {
    let factors = chromosome_factors(chrom)?;
    let card = |v: VarId| chrom.vars[v.0 as usize].card;

    // Split observed vs hidden.
    let mut obs: Vec<(VarId, usize)> = Vec::new();
    let mut hidden: Vec<VarId> = Vec::new();
    for (i, v) in chrom.vars.iter().enumerate() {
        let vid = VarId(i as u32);
        match row.get(i).copied().flatten() {
            Some(st) => obs.push((vid, st.min(v.card.saturating_sub(1)))),
            None => hidden.push(vid),
        }
    }

    // Condition every factor on observations (drops observed vars from scopes).
    let mut conditioned = Vec::with_capacity(factors.len());
    for f in &factors {
        conditioned.push(f.restrict(&obs)?);
    }

    // Sum out all remaining hidden vars using VE -> scalar.
    let out = variable_elimination(conditioned, &hidden, &card)?;
    Ok(out.logp().get(0).copied().unwrap_or(0.0))
}

/// Compute logZ (log-partition) by eliminating all variables.
pub fn logz(chrom: &PgmChromosome) -> Result<f32, String> {
    let factors = chromosome_factors(chrom)?;
    let card = |v: VarId| chrom.vars[v.0 as usize].card;
    let order: Vec<VarId> = (0..chrom.vars.len()).map(|i| VarId(i as u32)).collect();
    let out = variable_elimination(factors, &order, &card)?;
    Ok(out.logp().get(0).copied().unwrap_or(0.0))
}

/// Convenience: log P(evidence) under a normalized model.
#[inline]
pub fn logp_evidence(chrom: &PgmChromosome, row: &[Option<usize>]) -> Result<f32, String> {
    Ok(loglik_evidence(chrom, row)? - logz(chrom)?)
}

/// Convenience: negative mean log-likelihood over a dataset.
pub fn neg_mean_loglik(chrom: &PgmChromosome, data: &[Vec<Option<usize>>]) -> Result<f32, String> {
    let n = data.len().max(1) as f32;
    let mut sum = 0.0f32;
    for row in data {
        sum += logp_evidence(chrom, row)?;
    }
    Ok(-(sum / n))
}
