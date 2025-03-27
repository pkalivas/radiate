use radiate::{Distance, Genotype};

use crate::{Node, NodeType};

use super::{GraphChromosome, GraphNode};

pub struct GraphDistance;

impl<T> Distance<GraphChromosome<T>> for GraphDistance
where
    T: Clone + PartialEq + Default,
{
    fn distance(
        &self,
        one: &Genotype<GraphChromosome<T>>,
        two: &Genotype<GraphChromosome<T>>,
    ) -> f32 {
        let mut sum = 0.0;
        for (a, b) in one.iter().zip(two.iter()) {
            let one_stats = graph_stats(a.as_ref());
            let two_stats = graph_stats(b.as_ref());
            sum += l2_distance(&one_stats, &two_stats);
        }
        sum
    }
}

struct GraphStats {
    size: f32,
    avg_in: f32,
    avg_out: f32,
    input_nodes: i32,
    output_nodes: i32,
    vertex_nodes: i32,
    edge_nodes: i32,
}

fn graph_stats<T>(g: &[GraphNode<T>]) -> GraphStats {
    let mut in_sum = 0;
    let mut out_sum = 0;
    let mut types = [0; 4];

    for node in g.iter() {
        in_sum += node.incoming().len();
        out_sum += node.outgoing().len();

        match node.node_type() {
            NodeType::Input => types[0] += 1,
            NodeType::Output => types[1] += 1,
            NodeType::Vertex => types[2] += 1,
            NodeType::Edge => types[3] += 1,
            _ => {}
        }
    }

    let len = g.len().max(1) as f32;
    GraphStats {
        size: g.len() as f32,
        avg_in: in_sum as f32 / len,
        avg_out: out_sum as f32 / len,
        input_nodes: types[0],
        output_nodes: types[1],
        vertex_nodes: types[2],
        edge_nodes: types[3],
    }
}

fn l2_distance(a: &GraphStats, b: &GraphStats) -> f32 {
    let diffs = [
        (a.size - b.size).powi(2),
        (a.avg_in - b.avg_in).powi(2),
        (a.avg_out - b.avg_out).powi(2),
        ((a.input_nodes - b.input_nodes) as f32).powi(2),
        ((a.output_nodes - b.output_nodes) as f32).powi(2),
        ((a.vertex_nodes - b.vertex_nodes) as f32).powi(2),
        ((a.edge_nodes - b.edge_nodes) as f32).powi(2),
    ];
    diffs.iter().sum::<f32>().sqrt()
}
