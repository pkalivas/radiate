use crate::architects::node_collections::tracer::Tracer;
use crate::architects::schema::node_types::NodeType;
use crate::Node;

use super::super::graph::Graph;
use super::super::node_collection::NodeCollection;

pub struct GraphReducer<'a, T>
where
    T: Clone + PartialEq + Default,
{
    pub graph: &'a Graph<T>,
    pub tracers: Vec<Tracer<T>>,
    pub order: Vec<usize>,
    pub outputs: Vec<T>,
}

impl<'a, T> GraphReducer<'a, T>
where
    T: Clone + PartialEq + Default,
{
    pub fn new(graph: &'a Graph<T>) -> GraphReducer<'a, T> {
        let output_size = graph
            .iter()
            .filter(|node| node.node_type == NodeType::Output)
            .count();

        GraphReducer {
            graph,
            tracers: graph
                .iter()
                .map(|node| Tracer::new(GraphReducer::input_size(node)))
                .collect::<Vec<Tracer<T>>>(),
            order: Vec::with_capacity(graph.len()),
            outputs: vec![T::default(); output_size],
        }
    }

    #[inline]
    pub fn reduce(&mut self, inputs: &[T]) -> Vec<T> {
        if self.order.is_empty() {
            self.order = self
                .graph
                .topological_iter()
                .map(|node| node.index)
                .collect();
        }

        let mut output_index = 0;
        for index in &self.order {
            if let Some(node) = self.graph.get(*index) {
                if node.node_type == NodeType::Input {
                    self.tracers[node.index].add_input(inputs[node.index].clone());
                } else {
                    for incoming in &node.incoming {
                        let arg = self.tracers[*incoming]
                            .result
                            .clone()
                            .unwrap_or_else(|| T::default());
                        self.tracers[node.index].add_input(arg);
                    }
                }

                self.tracers[node.index].eval(&node);

                if node.node_type == NodeType::Output {
                    self.outputs[output_index] = self.tracers[node.index].result.clone().unwrap();
                    output_index += 1;
                }
            }
        }

        self.outputs.clone()
    }

    fn input_size(node: &Node<T>) -> usize {
        match node.node_type {
            NodeType::Input | NodeType::Link => 1,
            NodeType::Gate => node.value.arity() as usize,
            _ => node.incoming.len(),
        }
    }
}
