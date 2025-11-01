use crate::{Arity, Eval, NodeStore, Op, Tree, TreeNode};
use radiate_core::random_provider;
use std::sync::Arc;

impl<T> Op<T> {
    pub fn programs(&self) -> Option<&[TreeNode<Op<T>>]> {
        match self {
            Op::PGM(_, _, programs, _) => Some(programs.as_ref()),
            _ => None,
        }
    }

    pub fn with_programs<N>(&self, programs: impl Into<Vec<N>>) -> Self
    where
        N: Into<TreeNode<Op<T>>>,
        T: Clone,
    {
        match self {
            Op::PGM(name, arity, _, eval_fn) => Op::PGM(
                name,
                *arity,
                Arc::new(programs.into().into_iter().map(|p| p.into()).collect()),
                *eval_fn,
            ),
            _ => self.clone(),
        }
    }
}

impl Op<f32> {
    pub fn pgm<N>(name: &'static str, arity: impl Into<Arity>, programs: impl Into<Vec<N>>) -> Self
    where
        N: Into<TreeNode<Op<f32>>> + Clone,
    {
        let programs = programs
            .into()
            .into_iter()
            .map(|p| p.into())
            .collect::<Vec<TreeNode<Op<f32>>>>();

        let store = NodeStore::from(programs.clone());
        let arity = arity.into();

        let num_programs = match arity {
            Arity::Zero => 1,
            Arity::Exact(n) => n,
            Arity::Any => 2,
        };
        let pre_progs = (0..num_programs + 1)
            .filter_map(|_| Tree::with_depth(3, store.clone()).take_root())
            .enumerate()
            .map(|(i, node)| {
                if random_provider::bool(0.5) {
                    node
                } else {
                    programs[i % programs.len()].clone()
                }
            })
            .collect::<Vec<TreeNode<Op<f32>>>>();

        Op::PGM(
            name,
            arity,
            Arc::new(pre_progs),
            |inputs: &[f32], programs: &[TreeNode<Op<f32>>]| {
                let logits = (&programs[1..]).eval(inputs);

                if logits.is_empty() {
                    return 0.0;
                }

                let result = programs[0].eval(&logits);
                super::math::clamp(result)
            },
        )
    }

    /// PGM(LogSumExp): stable log(sum(exp(probs_i)))
    pub fn log_sum_exp<N>(programs: impl Into<Vec<N>>) -> Self
    where
        N: Into<TreeNode<Op<f32>>>,
    {
        Op::PGM(
            "log_sum_exp",
            Arity::Any,
            Arc::new(programs.into().into_iter().map(|p| p.into()).collect()),
            |inputs: &[f32], progs: &[TreeNode<Op<f32>>]| {
                if progs.is_empty() {
                    return 0.0;
                }

                let probabilities = progs.eval(inputs);

                let m = probabilities
                    .iter()
                    .copied()
                    .fold(f32::NEG_INFINITY, f32::max);
                let sum_exp = probabilities.iter().map(|v| (v - m).exp()).sum::<f32>();
                super::math::clamp(m + sum_exp.ln())
            },
        )
    }

    /// PGM(WeightedMean): probs interpreted as (w0, x0, w1, x1, ...)
    /// Uses raw sum of weights; returns 0 if denom ~ 0 or no pairs.
    pub fn weighted_mean<N>(programs: impl Into<Vec<N>>) -> Self
    where
        N: Into<TreeNode<Op<f32>>>,
    {
        Op::PGM(
            "weighted_mean",
            Arity::Any,
            Arc::new(programs.into().into_iter().map(|p| p.into()).collect()),
            |inputs: &[f32], progs: &[TreeNode<Op<f32>>]| {
                let probabilities = progs.eval(inputs);
                if probabilities.len() < 2 {
                    return 0.0;
                }
                let mut num = super::math::ZERO;
                let mut den = super::math::ZERO;

                let mut i = 0;
                while i + 1 < probabilities.len() {
                    let w = probabilities[i];
                    let x = probabilities[i + 1];
                    num += w * x;
                    den += w;
                    i += 2;
                }
                if den.abs() <= super::math::EPSILON {
                    0.0
                } else {
                    super::math::clamp(num / den)
                }
            },
        )
    }

    /// PGM(ClampNorm): L2-normalize probs, drop tiny components, then sum remaining.
    pub fn clamp_norm<N>(programs: impl Into<Vec<N>>) -> Self
    where
        N: Into<TreeNode<Op<f32>>>,
    {
        Op::PGM(
            "clamp_norm",
            Arity::Any,
            Arc::new(programs.into().into_iter().map(|p| p.into()).collect()),
            |inputs: &[f32], progs: &[TreeNode<Op<f32>>]| {
                let probabilities = progs.eval(inputs);
                if probabilities.is_empty() {
                    return 0.0;
                }

                let n = (probabilities.iter().map(|v| v * v).sum::<f32>())
                    .sqrt()
                    .max(super::math::EPSILON);
                let s = probabilities
                    .iter()
                    .map(|v| v / n)
                    .filter(|v| v.abs() >= super::math::THRESHOLD)
                    .sum::<f32>();
                super::math::clamp(s)
            },
        )
    }

    pub fn softmax<N>(programs: impl Into<Vec<N>>) -> Self
    where
        N: Into<TreeNode<Op<f32>>>,
    {
        Op::PGM(
            "softmax",
            Arity::Any,
            Arc::new(programs.into().into_iter().map(|p| p.into()).collect()),
            |inputs: &[f32], progs: &[TreeNode<Op<f32>>]| {
                let logits = progs.eval(inputs);
                if logits.is_empty() {
                    return 0.0;
                }

                let max_logit = logits.iter().copied().fold(f32::NEG_INFINITY, f32::max);
                let exp_sum: f32 = logits
                    .iter()
                    .map(|&x| super::math::clamp(x - max_logit).exp())
                    .sum();

                let softmaxed = logits
                    .iter()
                    .map(|&x| super::math::clamp((x - max_logit).exp() / exp_sum))
                    .collect::<Vec<f32>>();

                // Apply softmax normalization
                let sum = softmaxed.iter().sum::<f32>();
                let softmaxed = softmaxed
                    .into_iter()
                    .map(|x| super::math::clamp(x / sum))
                    .collect::<Vec<f32>>();

                let max_index = softmaxed
                    .iter()
                    .enumerate()
                    .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
                    .map(|(i, _)| i)
                    .unwrap_or(0);

                super::math::clamp(max_index as f32)
            },
        )
    }
}
