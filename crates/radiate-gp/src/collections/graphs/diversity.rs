use super::{GraphChromosome, GraphIterator};
use crate::{Node, Op};
use radiate_core::{Chromosome, Diversity, Genotype};
use std::cmp::Ordering;

const OP_MISMATCH_PENALTY: f32 = 0.3;

pub struct NeatDistance {
    excess: f32,
    disjoint: f32,
    weight_diff: f32,
    op_mismatch_penalty: f32,
}

impl NeatDistance {
    pub fn new(excess: f32, disjoint: f32, weight_diff: f32) -> Self {
        NeatDistance {
            excess,
            disjoint,
            weight_diff,
            op_mismatch_penalty: OP_MISMATCH_PENALTY,
        }
    }

    pub fn with_mismatch_penalty(mut self, penalty: f32) -> Self {
        self.op_mismatch_penalty = penalty;
        self
    }
}

impl Diversity<GraphChromosome<Op<f32>>> for NeatDistance {
    fn measure(
        &self,
        one: &Genotype<GraphChromosome<Op<f32>>>,
        two: &Genotype<GraphChromosome<Op<f32>>>,
    ) -> f32 {
        one.iter()
            .zip(two.iter())
            .map(|(a, b)| self.graph_distance_iter(a, b))
            .sum()
    }
}

impl NeatDistance {
    #[inline]
    fn graph_distance_iter(
        &self,
        a: &GraphChromosome<Op<f32>>,
        b: &GraphChromosome<Op<f32>>,
    ) -> f32 {
        let mut ita = a.iter_topological().peekable();
        let mut itb = b.iter_topological().peekable();

        let mut excess = 0.0f32;
        let mut disjoint = 0.0f32;
        let mut matching = 0.0f32;
        let mut weight_diff = 0.0f32;
        let mut op_mismatch_penalty = 0.0f32;

        let max_genes = a.len().max(b.len()) as f32;
        if max_genes == 0.0 {
            return 0.0;
        }
        let inv_max = 1.0 / max_genes;

        while ita.peek().is_some() || itb.peek().is_some() {
            match (ita.peek(), itb.peek()) {
                (Some(na), Some(nb)) => {
                    let na = *na;
                    let nb = *nb;

                    match na.index().cmp(&nb.index()) {
                        Ordering::Equal => {
                            if na.node_type() == nb.node_type() {
                                matching += 1.0;

                                match (na.value(), nb.value()) {
                                    (
                                        Op::MutableConst { value: va, .. },
                                        Op::MutableConst { value: vb, .. },
                                    ) => {
                                        weight_diff += (va - vb).abs();
                                    }
                                    (a_op, b_op) => {
                                        if a_op.name() != b_op.name() {
                                            op_mismatch_penalty += 1.0;
                                        }
                                    }
                                }
                            }
                            ita.next();
                            itb.next();
                        }
                        Ordering::Less => {
                            disjoint += 1.0;
                            ita.next();
                        }
                        Ordering::Greater => {
                            disjoint += 1.0;
                            itb.next();
                        }
                    }
                }
                (Some(_), None) => {
                    excess += 1.0;
                    ita.next();
                }
                (None, Some(_)) => {
                    excess += 1.0;
                    itb.next();
                }
                _ => break,
            }
        }

        let avg_weight_diff = if matching > 0.0 {
            weight_diff / matching
        } else {
            0.0
        };

        (self.excess * excess * inv_max)
            + (self.disjoint * disjoint * inv_max)
            + (self.weight_diff * avg_weight_diff)
            + (self.op_mismatch_penalty * op_mismatch_penalty)
    }
}
