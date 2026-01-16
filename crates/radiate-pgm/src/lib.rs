use radiate_core::random_provider;
use radiate_utils::Value;

mod alter;
mod chromosome;
mod codec;
mod factor;
mod fitness;
mod kernel;
mod var;

pub use alter::{PgmParamMutator, PgmScopeMutator};
pub use chromosome::{FactorGene, FactorKind, PgmChromosome};
pub use codec::PgmCodec;
pub use factor::{
    DiscreteFactor, chromosome_factors, gene_to_discrete, joint_factor, loglik_evidence,
    logp_evidence, logz, marginal_joint, marginal_ve, neg_mean_loglik,
};
pub use fitness::{PgmDataSet, PgmLogLik, PgmNll};
pub use kernel::{CptKernel, FactorKernel, IsingKernel};
pub use var::{VarId, VarSpec};

pub(crate) fn sample_scope(num_vars: usize, max_scope: usize) -> Vec<VarId> {
    let k = random_provider::range(1..max_scope.min(num_vars).max(1) + 1);

    let mut picked = Vec::with_capacity(k);
    while picked.len() < k {
        let v = VarId(random_provider::range(0..num_vars) as u32);
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

pub(crate) fn logp_table_shape(vars: &[VarSpec], scope: &[VarId]) -> Vec<usize> {
    scope
        .iter()
        .map(|&vid| vars[vid.0 as usize].card.max(1) as usize)
        .collect()
}

pub(crate) fn init_logp_table(shape: &[usize]) -> Value<f32> {
    Value::from((shape.to_vec(), |_| random_provider::range(-1.0..1.0)))
}

pub fn clamp_f32(x: f32, lo: f32, hi: f32) -> f32 {
    if x.is_nan() { 0.0 } else { x.clamp(lo, hi) }
}

#[inline]
pub fn logsumexp(xs: &[f32]) -> f32 {
    if xs.is_empty() {
        return f32::NEG_INFINITY;
    }
    let mut m = f32::NEG_INFINITY;
    for &x in xs {
        if x > m {
            m = x;
        }
    }
    if m.is_infinite() {
        return m;
    }
    let mut s = 0.0f32;
    for &x in xs {
        s += (x - m).exp();
    }
    m + s.ln()
}

/// log-space normalize a slice in-place so that exp(slice).sum() == 1.
#[inline]
pub fn log_normalize_in_place(row: &mut [f32]) {
    if row.is_empty() {
        return;
    }

    let mut m = f32::NEG_INFINITY;
    for &x in row.iter() {
        if x > m {
            m = x;
        }
    }

    if m.is_infinite() {
        // all -inf; keep as-is
        return;
    }

    let mut s = 0.0f32;
    for &x in row.iter() {
        s += (x - m).exp();
    }

    let lz = m + s.ln();
    for x in row.iter_mut() {
        *x -= lz;
    }
}

#[inline]
pub fn prod_usize(xs: &[usize]) -> usize {
    xs.iter().fold(1usize, |acc, &v| acc.saturating_mul(v))
}

/// A very small discrete-only VE evaluator:
/// - multiply all factors that mention elim var
/// - marginalize elim var
/// - put result back
///
/// For correctness validation and N <= ~10ish factors/vars.
pub fn variable_elimination(
    mut factors: Vec<DiscreteFactor>,
    elim_order: &[VarId],
    card: &impl Fn(VarId) -> usize,
) -> Result<DiscreteFactor, String> {
    for &z in elim_order {
        // split factors into those that contain z and those that don't
        let mut with = Vec::new();
        let mut without = Vec::new();

        for f in factors.into_iter() {
            if f.scope().contains(&z) {
                with.push(f);
            } else {
                without.push(f);
            }
        }

        // if no factor includes z, keep going
        if with.is_empty() {
            factors = without;
            continue;
        }

        // multiply them all
        let mut joint = with[0].clone();
        for f in with.iter().skip(1) {
            joint = joint.product(f, card)?;
        }

        // marginalize z out
        let reduced = joint.marginalize(&[z])?;

        without.push(reduced);
        factors = without;
    }

    // multiply remaining factors into one joint
    if factors.is_empty() {
        return Err("no factors".into());
    }
    let mut joint = factors[0].clone();
    for f in factors.iter().skip(1) {
        joint = joint.product(f, card)?;
    }
    Ok(joint)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::var::VarSpec;

    fn approx(a: f32, b: f32, eps: f32) -> bool {
        (a - b).abs() <= eps
    }

    #[test]
    fn ve_matches_bruteforce_small_chain() {
        // A(2) -> B(2) -> C(2)
        let a = VarSpec::new(0, 2);
        let b = VarSpec::new(1, 2);
        let c = VarSpec::new(2, 2);

        let card = |v: VarId| match v.0 {
            0 => 2,
            1 => 2,
            2 => 2,
            _ => 0,
        };

        // Prior P(A): logits [0, 1] then normalized (as CPT with "child" A and no parents)
        let mut p_a = DiscreteFactor::new(vec![a], vec![0.0, 1.0]).unwrap();
        // normalize across its only axis by treating it as "child"
        p_a.normalize_rows(VarId(0)).unwrap();

        // CPT P(B|A) scope [A,B]
        let mut p_ba = DiscreteFactor::new(
            vec![a, b],
            vec![
                2.0, 0.0, // A=0: B=0..1
                0.0, 2.0, // A=1
            ],
        )
        .unwrap();
        p_ba.normalize_rows(VarId(1)).unwrap();

        // CPT P(C|B) scope [B,C]
        let mut p_cb = DiscreteFactor::new(
            vec![b, c],
            vec![
                2.0, 0.0, // B=0
                0.0, 2.0, // B=1
            ],
        )
        .unwrap();
        p_cb.normalize_rows(VarId(2)).unwrap();

        // We want marginal P(C) by eliminating A,B.
        let ve = variable_elimination(
            vec![p_a.clone(), p_ba.clone(), p_cb.clone()],
            &[VarId(0), VarId(1)],
            &card,
        )
        .unwrap();
        assert_eq!(ve.scope(), &[VarId(2)]);

        // brute force: sum_{a,b} P(a)P(b|a)P(c|b)
        for ci in 0..2 {
            let mut acc = Vec::new();
            for ai in 0..2 {
                for bi in 0..2 {
                    let lp = p_a.log_value_aligned(&[ai])
                        + p_ba.log_value_aligned(&[ai, bi])
                        + p_cb.log_value_aligned(&[bi, ci]);
                    acc.push(lp);
                }
            }
            let want = crate::logsumexp(&acc);
            let got = ve.log_value_aligned(&[ci]);
            assert!(approx(got, want, 1e-5), "c={ci} got={got} want={want}");
        }
    }
}

// use radiate_pgm_skeleton::kernels::{CptKernel, IsingKernel, FactorKernel};
// use radiate_pgm_skeleton::ve::variable_elimination;
// use radiate_pgm_skeleton::var::{VarSpec, VarId};

// fn main() -> Result<(), String> {
//     let a = VarSpec::new(0, 2);
//     let b = VarSpec::new(1, 2);

//     let card = |v: VarId| match v.0 { 0 => 2, 1 => 2, _ => 0 };

//     // Build an Ising factor on (A,B)
//     let ising = IsingKernel { a, b };
//     let f_ab = ising.build(&[0.0, 0.1, 0.0, 0.2, 0.5, -0.5])?;

//     // Build a CPT P(A) as a CPT with no parents (scope [A])
//     let prior = CptKernel { parents: vec![], child: a };
//     let p_a = prior.build(&[0.0, 1.0])?;

//     // Compute marginal P(B) by eliminating A from P(A)*f(A,B)
//     let out = variable_elimination(vec![p_a, f_ab], &[VarId(0)], &card)?;
//     println!("scope={:?}, logp={:?}", out.scope(), out.logp());
//     Ok(())
// }
