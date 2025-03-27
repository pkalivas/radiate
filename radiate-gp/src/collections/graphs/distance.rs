use super::{GraphChromosome, GraphIterator, GraphNode};
use crate::{Node, NodeType, Op};
use radiate::{Distance, Genotype};
use std::cmp::Ordering;

pub struct NeatDistance {
    distance: f32,
    excess: f32,
    disjoint: f32,
    weight_diff: f32,
}

impl NeatDistance {
    pub fn new(distance: f32, excess: f32, disjoint: f32, weight_diff: f32) -> Self {
        Self {
            distance,
            excess,
            disjoint,
            weight_diff,
        }
    }
}

impl Distance<GraphChromosome<Op<f32>>> for NeatDistance {
    fn threshold(&self) -> f32 {
        self.distance
    }

    fn distance(
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

// fn distance(
//     &self,
//     one: &Genotype<GraphChromosome<T>>,
//     two: &Genotype<GraphChromosome<T>>,
// ) -> f32 {
//     let mut sum = 0.0;
//     for (a, b) in one.iter().zip(two.iter()) {
//         let one_stats = graph_stats(a.as_ref());
//         let two_stats = graph_stats(b.as_ref());
//         sum += l2_distance(&one_stats, &two_stats);
//     }
//     sum
// }
// struct GraphStats {
//     size: f32,
//     avg_in: f32,
//     avg_out: f32,
//     input_nodes: i32,
//     output_nodes: i32,
//     vertex_nodes: i32,
//     edge_nodes: i32,
// }

// fn graph_stats<T>(g: &[GraphNode<T>]) -> GraphStats {
//     let mut in_sum = 0;
//     let mut out_sum = 0;
//     let mut types = [0; 4];

//     for node in g.iter() {
//         in_sum += node.incoming().len();
//         out_sum += node.outgoing().len();

//         match node.node_type() {
//             NodeType::Input => types[0] += 1,
//             NodeType::Output => types[1] += 1,
//             NodeType::Vertex => types[2] += 1,
//             NodeType::Edge => types[3] += 1,
//             _ => {}
//         }
//     }

//     let len = g.len().max(1) as f32;
//     GraphStats {
//         size: g.len() as f32,
//         avg_in: in_sum as f32 / len,
//         avg_out: out_sum as f32 / len,
//         input_nodes: types[0],
//         output_nodes: types[1],
//         vertex_nodes: types[2],
//         edge_nodes: types[3],
//     }
// }

// fn l2_distance(a: &GraphStats, b: &GraphStats) -> f32 {
//     let diffs = [
//         (a.size - b.size).powi(2),
//         (a.avg_in - b.avg_in).powi(2),
//         (a.avg_out - b.avg_out).powi(2),
//         ((a.input_nodes - b.input_nodes) as f32).powi(2),
//         ((a.output_nodes - b.output_nodes) as f32).powi(2),
//         ((a.vertex_nodes - b.vertex_nodes) as f32).powi(2),
//         ((a.edge_nodes - b.edge_nodes) as f32).powi(2),
//     ];
//     diffs.iter().sum::<f32>().sqrt()
// }
