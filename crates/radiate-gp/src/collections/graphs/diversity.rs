use super::{GraphChromosome, GraphIterator, GraphNode};
use crate::{Node, Op};
use radiate_core::{Diversity, Genotype};
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
            .map(|(a, b)| {
                self.graph_distance(
                    a.iter_topological().collect::<Vec<_>>().as_slice(),
                    b.iter_topological().collect::<Vec<_>>().as_slice(),
                )
            })
            .sum()
    }
}

impl NeatDistance {
    fn graph_distance(
        &self,
        nodes_a: &[&GraphNode<Op<f32>>],
        nodes_b: &[&GraphNode<Op<f32>>],
    ) -> f32 {
        let mut idx_a = 0;
        let mut idx_b = 0;

        let mut excess = 0.0;
        let mut disjoint = 0.0;
        let mut matching = 0.0;
        let mut weight_diff = 0.0;
        let mut op_mismatch_penalty = 0.0;

        while idx_a < nodes_a.len() || idx_b < nodes_b.len() {
            match (nodes_a.get(idx_a), nodes_b.get(idx_b)) {
                (Some(na), Some(nb)) => match na.index().cmp(&nb.index()) {
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
                        idx_a += 1;
                        idx_b += 1;
                    }
                    Ordering::Less => {
                        disjoint += 1.0;
                        idx_a += 1;
                    }
                    Ordering::Greater => {
                        disjoint += 1.0;
                        idx_b += 1;
                    }
                },
                (Some(_), None) => {
                    excess += 1.0;
                    idx_a += 1;
                }
                (None, Some(_)) => {
                    excess += 1.0;
                    idx_b += 1;
                }
                _ => break,
            }
        }

        let avg_weight_diff = if matching > 0.0 {
            weight_diff / matching
        } else {
            0.0
        };
        let max_genes = nodes_a.len().max(nodes_b.len()) as f32;

        (self.excess * excess / max_genes)
            + (self.disjoint * disjoint / max_genes)
            + (self.weight_diff * avg_weight_diff)
            + (self.op_mismatch_penalty * op_mismatch_penalty)
    }
}
