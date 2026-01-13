use std::sync::Arc;

use crate::{
    Arity, Eval, Op,
    ops::{
        OpValue,
        math::{ONE, TENTH, ZERO, clamp},
        op_names,
    },
};
use radiate_core::{Value, random_provider};
use radiate_utils::Shape;

impl Op<f32> {
    /// ND array lookup operation.
    ///
    /// Uses inputs as indices into an N-dimensional array stored in the OpValue.
    /// Out-of-bounds indices are clamped to the valid range.
    pub fn nd_array(shape: impl Into<Shape>) -> Self {
        let shape = shape.into();

        Op::Value(
            op_names::ND_ARRAY,
            Arity::Exact(shape.dimensions()),
            Self::categorical_table(shape.clone()),
            |inputs: &[f32], val: &Value<f32>| match val {
                Value::Array {
                    shape,
                    values,
                    strides,
                } => {
                    let idx = (0..inputs.len()).fold(0usize, |index, i| {
                        let stride = strides.stride_at(i);
                        let dim = shape.dim_at(i).max(1);

                        let val = (inputs[i].round() as isize).max(0) as usize;

                        index.saturating_add(val.min(dim - 1).saturating_mul(stride))
                    });

                    values.get(idx).copied().map(clamp).unwrap_or(ZERO)
                }
                _ => ZERO,
            },
        )
    }

    /// A conditional categorical table returning log P(child | parents).
    ///
    /// Shape dims: [d0, d1, ..., d_{k-1}, child_states]
    /// Inputs:     [p0, p1, ..., p_{k-1}, child]
    pub fn logprob_table(shape: impl Into<Shape>) -> Self {
        let shape = shape.into();
        let rank = shape.rank();

        // Evaluate: log softmax along child axis at the given parent configuration
        let eval = |inputs: &[f32], val: &Value<f32>| -> f32 {
            let Value::Array {
                values,
                shape,
                strides,
            } = val
            else {
                return ZERO;
            };

            // Convert inputs -> clamped indices
            let mut idxs = vec![0usize; shape.rank()];
            for i in 0..shape.rank() {
                let dim = shape.dim_at(i).max(1);
                let mut v = inputs[i].round() as isize;
                if v < 0 {
                    v = 0;
                }
                let v = v as usize;
                idxs[i] = v.min(dim - 1);
            }

            let child_axis = shape.rank() - 1;
            let child = idxs[child_axis];

            // Base offset with child fixed to 0
            let mut base = 0usize;
            for i in 0..child_axis {
                base = base.saturating_add(idxs[i].saturating_mul(strides.stride_at(i)));
            }

            // Find max logit for stable logsumexp
            let mut max_logit = f32::NEG_INFINITY;
            let child_states = shape.dim_at(child_axis).max(1);
            for k in 0..child_states {
                let pos = base.saturating_add(k.saturating_mul(strides.stride_at(child_axis)));
                let s = values.get(pos).copied().unwrap_or(0.0);
                if s > max_logit {
                    max_logit = s;
                }
            }
            if !max_logit.is_finite() {
                return ZERO;
            }

            // logsumexp
            let mut sum_exp = 0.0f32;
            for k in 0..child_states {
                let pos = base.saturating_add(k.saturating_mul(strides.stride_at(child_axis)));
                let s = values.get(pos).copied().unwrap_or(0.0);
                sum_exp += (s - max_logit).exp();
            }
            let lse = max_logit + sum_exp.ln();

            // selected child logit - logsumexp = log probability
            let child_pos =
                base.saturating_add(child.saturating_mul(strides.stride_at(child_axis)));
            let child_logit = values.get(child_pos).copied().unwrap_or(0.0);

            clamp(child_logit - lse)
        };

        Op::Value(
            op_names::LOGPROB_TABLE,
            Arity::Exact(rank),
            Self::categorical_table(shape.clone()),
            eval,
        )
    }

    fn categorical_table(shape: Shape) -> OpValue<f32> {
        let init = Value::from((shape.clone(), |_| random_provider::range(-ONE..ONE)));

        let supplier = |value: &Value<f32>| match value {
            Value::Array { shape: sha, .. } => {
                Value::from((sha.clone(), |_| random_provider::range(-ONE..ONE)))
            }
            _ => Value::from((value.shape().unwrap().clone(), |_| {
                random_provider::range(-ONE..ONE)
            })),
        };

        let modifier = |current: &mut Value<f32>| {
            if let Value::Array { values, .. } = current {
                let values = Arc::make_mut(values);
                random_provider::with_rng(|rng| {
                    let n = values.len().max(1);
                    let k = (n / 10 + 1).min(n);
                    let idxs = rng.sample_indices(0..n, k);

                    for &idx in &idxs {
                        let diff = (rng.range(-ONE..ONE)) * TENTH;
                        values[idx] = clamp(values[idx] + diff);
                    }
                });
            }
        };

        OpValue::new(init, supplier, modifier)
    }

    pub fn gauss1() -> Self {
        Op::Value(
            op_names::GAUSS1,
            Arity::Exact(1),
            Self::gauss1_params(),
            |inputs: &[f32], val: &Value<f32>| -> f32 {
                let x = inputs.get(0).copied().unwrap_or(0.0);

                let Value::Array { values, .. } = val else {
                    return 0.0;
                };

                if values.len() < 2 {
                    return 0.0;
                }

                let mu = values[0];
                let log_sigma = values[1].clamp(-8.0, 4.0);
                let sigma = log_sigma.exp().max(1e-6);

                // log N(x|mu,sigma) = -0.5*((x-mu)/sigma)^2 - log(sigma) - 0.5*log(2pi)
                let z = (x - mu) / sigma;
                let ll = -0.5 * z * z - log_sigma - 0.9189385332; // 0.5*ln(2π)
                ll
            },
        )
    }

    fn gauss1_params() -> OpValue<f32> {
        // params = [mu, log_sigma]
        let init = Value::from((2, |_| random_provider::range(-1.0..1.0)));

        let supplier = |v: &Value<f32>| match v {
            Value::Array { .. } => Value::from((2, |_| random_provider::range(-1.0..1.0))),
            _ => Value::from((2, |_| random_provider::range(-1.0..1.0))),
        };

        let modifier = |current: &mut Value<f32>| {
            if let Value::Array { values, .. } = current {
                let values = Arc::make_mut(values);
                // jitter mu
                values[0] += random_provider::range(-0.1..0.1);
                // jitter log_sigma (smaller)
                values[1] += random_provider::range(-0.05..0.05);
                values[1] = values[1].clamp(-8.0, 4.0);
            }
        };

        OpValue::new(init, supplier, modifier)
    }

    pub fn gauss_lin2() -> Self {
        Op::Value(
            op_names::GAUSS_LIN2,
            Arity::Exact(2),
            Self::gauss_lin2_params(),
            |inputs: &[f32], val: &Value<f32>| -> f32 {
                let x = inputs.get(0).copied().unwrap_or(0.0);
                let y = inputs.get(1).copied().unwrap_or(0.0);

                let Value::Array { values, .. } = val else {
                    return 0.0;
                };
                if values.len() < 3 {
                    return 0.0;
                }

                let a = values[0];
                let b = values[1];
                let log_sigma = values[2].clamp(-8.0, 4.0);
                let sigma = log_sigma.exp().max(1e-6);

                let mu = a * x + b;
                let z = (y - mu) / sigma;
                -0.5 * z * z - log_sigma - 0.9189385332
            },
        )
    }

    fn gauss_lin2_params() -> OpValue<f32> {
        // params = [a, b, log_sigma]
        let init = Value::from((3, |_| random_provider::range(-1.0..1.0)));

        let supplier = |_: &Value<f32>| Value::from((3, |_| random_provider::range(-1.0..1.0)));

        let modifier = |current: &mut Value<f32>| {
            if let Value::Array { values, .. } = current {
                let values = Arc::make_mut(values);
                values[0] += random_provider::range(-0.05..0.05); // a
                values[1] += random_provider::range(-0.10..0.10); // b
                values[2] += random_provider::range(-0.05..0.05); // log_sigma
                values[2] = values[2].clamp(-8.0, 4.0);
            }
        };

        OpValue::new(init, supplier, modifier)
    }
}

#[inline]
pub fn markov_loglik(table: &Op<f32>, states: &[usize], k: usize) -> f32 {
    let mut ll = 0.0;
    for t in 0..states.len().saturating_sub(1) {
        let prev = (states[t].min(k - 1)) as f32;
        let next = (states[t + 1].min(k - 1)) as f32;
        ll += table.eval(&[prev, next]); // log P(next | prev)
    }

    ll
}

#[inline]
pub fn logsumexp(xs: &[f32]) -> f32 {
    let m = xs.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let s = xs.iter().map(|x| (x - m).exp()).sum::<f32>();

    m + s.ln()
}

#[inline]
pub fn argmax_from_logprob_table(op: &Op<f32>, parents: &[f32]) -> usize {
    // We need child_states and rank from the underlying Value::Array
    let (rank, child_states) = match op.value() {
        Some(Value::Array { shape, .. }) => {
            let rank = shape.rank();
            let child_states = shape.dim_at(rank - 1).max(1);
            (rank, child_states)
        }
        _ => return 0,
    };

    let mut inputs = Vec::with_capacity(rank);
    inputs.extend_from_slice(parents);

    let mut best_k = 0usize;
    let mut best_lp = f32::NEG_INFINITY;

    for k in 0..child_states {
        inputs.push(k as f32);
        let lp = op.eval(&inputs);
        inputs.pop();

        if lp > best_lp {
            best_lp = lp;
            best_k = k;
        }
    }

    best_k
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Eval;
    use crate::Op;
    use std::f32;

    #[test]
    fn sanity_prob_sums_to_one() {
        let k = 5;
        let op = Op::<f32>::logprob_table((k, k));

        for prev in 0..k {
            let mut logps = vec![];
            for next in 0..k {
                logps.push(op.eval(&[prev as f32, next as f32]));
            }
            let lse = logsumexp(&logps);
            assert!(lse.abs() < 1e-4, "prev={prev}, lse={lse}");
        }
    }

    #[test]
    fn test_nd_array_op() {
        let shape = Shape::from((3, 4));
        let op = Op::<f32>::nd_array(shape.clone());

        for row in 0..shape.dim_at(0) {
            for col in 0..shape.dim_at(1) {
                let indices = [row as f32, col as f32];
                let val = op.eval(&indices);
                println!("Value at ({},{}) : {}", row, col, val);
                assert!(val >= -1.0 && val <= 1.0);
            }
        }
    }
}
