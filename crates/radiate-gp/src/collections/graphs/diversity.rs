use super::{GraphChromosome, GraphIterator, GraphNode};
use crate::{Node, NodeType, Op};
use radiate_core::{Diversity, Genotype};
use std::cmp::Ordering;

pub struct NeatDistance {
    excess: f32,
    disjoint: f32,
    weight_diff: f32,
}

impl NeatDistance {
    pub fn new(excess: f32, disjoint: f32, weight_diff: f32) -> Self {
        NeatDistance {
            excess,
            disjoint,
            weight_diff,
        }
    }
}

impl Diversity<GraphChromosome<Op<f32>>> for NeatDistance {
    fn measure(
        &self,
        one: &Genotype<GraphChromosome<Op<f32>>>,
        two: &Genotype<GraphChromosome<Op<f32>>>,
    ) -> f32 {
        let mut excess = 0.0;
        let mut disjoint = 0.0;
        let mut matching = 0.0;
        let mut weight_diff = 0.0;

        let mut distance = 0.0;

        for (a, b) in one.iter().zip(two.iter()) {
            let mut idx_a = 0;
            let mut idx_b = 0;

            let nodes_a = a.iter_topological().collect::<Vec<&GraphNode<Op<f32>>>>();
            let nodes_b = b.iter_topological().collect::<Vec<&GraphNode<Op<f32>>>>();

            while idx_a < nodes_a.len() || idx_b < nodes_b.len() {
                match (nodes_a.get(idx_a), nodes_b.get(idx_b)) {
                    (Some(na), Some(nb)) => match na.index().cmp(&nb.index()) {
                        Ordering::Equal => {
                            if NodeType::Edge == na.node_type() && nb.node_type() == NodeType::Edge
                            {
                                matching += 1.0;

                                if let (
                                    Op::MutableConst { value: va, .. },
                                    Op::MutableConst { value: vb, .. },
                                ) = (na.value(), nb.value())
                                {
                                    weight_diff += (va - vb).abs();
                                }
                            } else if NodeType::Vertex == na.node_type()
                                && nb.node_type() == NodeType::Vertex
                            {
                                matching += 1.0;

                                if let (
                                    Op::MutableConst { value: va, .. },
                                    Op::MutableConst { value: vb, .. },
                                ) = (na.value(), nb.value())
                                {
                                    weight_diff += (va - vb).abs();
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

            distance += (self.excess * excess / max_genes)
                + (self.disjoint * disjoint / max_genes)
                + (self.weight_diff * avg_weight_diff);
        }

        distance
    }
}
