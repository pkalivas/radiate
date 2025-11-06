use crate::{Arity, Eval, Node, NodeStore, Op, Tree, TreeNode, ops::op_names};
use radiate_core::random_provider;
use std::{fmt::Debug, sync::Arc};

pub(super) const THRESHOLD: f32 = 1e-3_f32;
pub(super) const EPSILON: f32 = 1e-12_f32;

impl<T> Op<T> {
    pub fn programs(&self) -> Option<&[TreeNode<Op<T>>]> {
        match self {
            Op::PGM(_, _, programs, _) => Some(programs.as_ref()),
            _ => None,
        }
    }

    pub fn programs_mut(&mut self) -> Option<&mut Vec<TreeNode<Op<T>>>>
    where
        T: Clone,
    {
        match self {
            Op::PGM(_, _, programs, _) => Some(Arc::make_mut(programs)),
            _ => None,
        }
    }

    // Program seeding utility used by defaults
    pub(self) fn seed_programs(
        arity: Arity,
        store: &NodeStore<Op<T>>,
        seeds: &[TreeNode<Op<T>>],
        depth: usize,
        keep_ratio: f32,
        add_tail: bool,
    ) -> Arc<Vec<TreeNode<Op<T>>>>
    where
        T: Clone + Default + Debug,
    {
        let n = match arity {
            Arity::Zero => 1,
            Arity::Exact(k) => k,
            Arity::Any => 2,
        };

        // First program is the “head”; the rest are “tail” (inputs → probs/logits)
        let num_progs = if add_tail { n + 1 } else { n };

        let mut out = Vec::with_capacity(num_progs);
        for i in 0..num_progs {
            let from_seed = !seeds.is_empty() && !random_provider::bool(1.0 - keep_ratio);
            if from_seed {
                let mut current = seeds[i % seeds.len()].clone();
                if current.value().arity() != Arity::Zero {
                    Tree::repair_node(&mut current, store);
                }

                out.push(current);
            } else {
                if let Some(node) = Tree::with_depth(depth, store.clone()).take_root() {
                    out.push(node);
                }
            }
        }

        Arc::new(out)
    }
}

impl Op<f32> {
    pub fn pgm_with<N>(
        name: &'static str,
        arity: impl Into<Arity>,
        seeds: impl Into<Vec<N>>,
        depth: usize,
        keep_ratio: f32,
        add_tail: bool,
        eval: fn(&[f32], &[TreeNode<Op<f32>>]) -> f32,
    ) -> Op<f32>
    where
        N: Into<TreeNode<Op<f32>>>,
    {
        let arity = arity.into();
        let seeds = seeds
            .into()
            .into_iter()
            .map(|s| s.into())
            .collect::<Vec<TreeNode<Op<f32>>>>();
        let store = NodeStore::from(seeds.to_vec());
        let programs = Self::seed_programs(arity, &store, &seeds, depth, keep_ratio, add_tail);

        Op::PGM(name, arity, programs, eval)
    }

    pub fn pgm<N>(name: &'static str, arity: impl Into<Arity>, programs: impl Into<Vec<N>>) -> Self
    where
        N: Into<TreeNode<Op<f32>>> + Clone,
    {
        Self::pgm_with(
            name,
            arity,
            programs,
            3,
            0.5,
            true,
            |inputs: &[f32], programs: &[TreeNode<Op<f32>>]| {
                let logits = (&programs[1..]).eval(inputs);
                if logits.is_empty() {
                    return 0.0;
                }

                let probabilities = Self::stable_softmax(&logits);
                let result = programs[0].eval(&probabilities);
                super::math::clamp(result)
            },
        )
    }

    /// PGM(LogSumExp): stable log(sum(exp(probs_i)))
    pub fn log_sum_exp<N>(programs: impl Into<Vec<N>>) -> Self
    where
        N: Into<TreeNode<Op<f32>>>,
    {
        Self::pgm_with(
            op_names::LOG_SUM_EXP,
            Arity::Any,
            programs,
            3,
            0.5,
            false,
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
        Self::pgm_with(
            op_names::WEIGHTED_MEAN,
            Arity::Any,
            programs,
            3,
            0.5,
            false,
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
                if den.abs() <= EPSILON {
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
        Self::pgm_with(
            op_names::CLAMP_NORM,
            Arity::Any,
            programs,
            3,
            0.5,
            false,
            |inputs: &[f32], progs: &[TreeNode<Op<f32>>]| {
                let probabilities = progs.eval(inputs);
                if probabilities.is_empty() {
                    return 0.0;
                }

                let n = (probabilities
                    .iter()
                    .map(|v| super::math::clamp(*v * *v))
                    .sum::<f32>())
                .sqrt()
                .max(EPSILON);

                let s = probabilities
                    .iter()
                    .map(|v| super::math::clamp(*v / n))
                    .filter(|v| v.abs() >= THRESHOLD)
                    .sum::<f32>();

                super::math::clamp(s)
            },
        )
    }

    // Default: Softmax of tail, return argmax (index)
    pub fn softmax_argmax<N>(seeds: impl Into<Vec<N>>) -> Op<f32>
    where
        N: Into<TreeNode<Op<f32>>>,
    {
        Self::pgm_with(
            op_names::SOFTMAX_ARGMAX,
            Arity::Any,
            seeds,
            3,
            0.5,
            false,
            |inputs: &[f32], progs: &[TreeNode<Op<f32>>]| {
                let logits = progs.eval(inputs);
                if logits.is_empty() {
                    return 0.0;
                }

                let sm = Self::stable_softmax(&logits);
                let idx = sm
                    .iter()
                    .enumerate()
                    .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
                    .map(|(i, _)| i)
                    .unwrap_or(0);

                super::math::clamp(idx as f32)
            },
        )
    }

    // Default: Attention-like weighted sum with softmax weights over even positions
    pub fn attention_sum<N>(seeds: impl Into<Vec<N>>) -> Op<f32>
    where
        N: Into<TreeNode<Op<f32>>>,
    {
        Self::pgm_with(
            op_names::ATTENTION_SUM,
            Arity::Any,
            seeds,
            2,
            0.5,
            false,
            |inputs, progs| {
                let vals = progs.eval(inputs);
                if vals.len() < 2 {
                    return 0.0;
                }
                // Interpret as (logit_w0, x0, logit_w1, x1, ...)
                let logits = vals.iter().copied().step_by(2).collect::<Vec<f32>>();
                let xs = vals
                    .iter()
                    .copied()
                    .skip(1)
                    .step_by(2)
                    .collect::<Vec<f32>>();

                if logits.is_empty() || xs.is_empty() {
                    return 0.0;
                }

                let ws = Self::stable_softmax(&logits);
                let m = ws.iter().zip(xs.iter()).map(|(w, x)| w * x).sum::<f32>();

                super::math::clamp(m)
            },
        )
    }

    pub fn pgm_set<N>(seeds: impl Into<Vec<N>>) -> Vec<Op<f32>>
    where
        N: Into<TreeNode<Op<f32>>>,
    {
        let seeds = seeds
            .into()
            .into_iter()
            .map(|s| s.into())
            .collect::<Vec<TreeNode<Op<f32>>>>();

        vec![
            Op::log_sum_exp(seeds.clone()),
            Op::weighted_mean(seeds.clone()),
            Op::clamp_norm(seeds.clone()),
            Op::softmax_argmax(seeds.clone()),
            Op::attention_sum(seeds.clone()),
        ]
    }

    #[inline]
    fn stable_softmax(xs: &[f32]) -> Vec<f32> {
        if xs.is_empty() {
            return vec![];
        }

        let m = xs.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        let exps = xs
            .iter()
            .map(|&x| super::math::clamp((x - m).exp()))
            .collect::<Vec<f32>>();

        let s = exps.iter().sum::<f32>().max(EPSILON);

        exps.into_iter()
            .map(|e| super::math::clamp(e / s))
            .collect()
    }
}
