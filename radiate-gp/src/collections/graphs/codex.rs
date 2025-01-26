// use crate::collections::graphs::aggregate::GraphAggregate;
// use crate::collections::graphs::builder::GraphBuilder;
// use crate::collections::{Graph, GraphNode, NodeType};
// use crate::graphs::chromosome::GraphChromosome;
// use crate::ops::Op;
// use crate::{Builder, Factory, NodeCell};
// use radiate::{Chromosome, Codex, Gene, Genotype};
// use std::sync::{Arc, RwLock};

// use super::CellStore;

// pub struct GraphCodex<C: NodeCell> {
//     store: Arc<RwLock<CellStore<C>>>,
//     graph: Option<Graph<C>>,
// }

// impl<C> GraphCodex<C>
// where
//     C: NodeCell + Clone + Default,
// {
//     pub fn new() -> Self {
//         GraphCodex {
//             store: Arc::new(RwLock::new(CellStore::new())),
//             graph: None,
//         }
//     }

//     pub fn from_graph(graph: Graph<C>, factory: &CellStore<C>) -> Self {
//         GraphCodex {
//             store: Arc::new(RwLock::new(factory.clone())),
//             graph: Some(graph),
//         }
//     }

//     pub fn set_graph<F>(mut self, node_fn: F) -> Self
//     where
//         F: Fn(&GraphBuilder<C>, GraphAggregate<C>) -> Graph<C>,
//     {
//         let graph = node_fn(
//             &GraphBuilder::new((*self.store.read().unwrap()).clone()),
//             GraphAggregate::new(),
//         );

//         self.graph = Some(graph);
//         self
//     }

//     pub fn with_vertices(self, vertices: Vec<C>) -> Self {
//         self.set_values(NodeType::Vertex, vertices);
//         self
//     }

//     pub fn with_edges(self, edges: Vec<C>) -> Self {
//         self.set_values(NodeType::Edge, edges);
//         self
//     }

//     pub fn with_inputs(self, inputs: Vec<C>) -> Self {
//         self.set_values(NodeType::Input, inputs);
//         self
//     }

//     pub fn with_output(self, outputs: C) -> Self {
//         self.set_values(NodeType::Output, vec![outputs]);
//         self
//     }

//     fn set_values(&self, node_type: NodeType, values: Vec<C>) {
//         let mut factory = self.store.write().unwrap();
//         factory.add_values(node_type, values);
//     }
// }

// impl GraphCodex<Op<f32>> {
//     pub fn lstm(input_size: usize, hidden_size: usize, output_size: usize) -> Self {
//         let store = CellStore::regressor(input_size);
//         let nodes = GraphBuilder::<Op<f32>>::new(store.clone())
//             .lstm(input_size, hidden_size, output_size, Op::linear())
//             .build();
//         GraphCodex::<Op<f32>>::from_graph(nodes, &store)
//     }

//     pub fn gru(input_size: usize, output_size: usize, hidden_size: usize) -> Self {
//         let store = CellStore::regressor(input_size);
//         let nodes = GraphBuilder::<Op<f32>>::new(store.clone())
//             .gru(input_size, output_size, hidden_size, Op::linear())
//             .build();
//         GraphCodex::<Op<f32>>::from_graph(nodes, &store)
//     }

//     pub fn acyclic(input_size: usize, output_size: usize) -> Self {
//         let store = CellStore::regressor(input_size);
//         let nodes = GraphBuilder::<Op<f32>>::default()
//             .with_store(store.clone())
//             .acyclic(input_size, output_size, Op::linear())
//             .build();
//         GraphCodex::<Op<f32>>::from_graph(nodes, &store)
//     }

//     pub fn cyclic(input_size: usize, output_size: usize) -> Self {
//         let store = CellStore::regressor(input_size);
//         let nodes = GraphBuilder::<Op<f32>>::new(store.clone())
//             .cyclic(input_size, output_size, Op::linear())
//             .build();
//         GraphCodex::<Op<f32>>::from_graph(nodes, &store)
//     }

//     pub fn weighted_acyclic(input_size: usize, output_size: usize) -> Self {
//         let store = CellStore::regressor(input_size);
//         let nodes = GraphBuilder::<Op<f32>>::new(store.clone())
//             .weighted_acyclic(input_size, output_size, Op::linear())
//             .build();
//         GraphCodex::<Op<f32>>::from_graph(nodes, &store)
//     }

//     pub fn weighted_cyclic(input_size: usize, output_size: usize, memory_size: usize) -> Self {
//         let store = CellStore::regressor(input_size);
//         let nodes = GraphBuilder::<Op<f32>>::new(store.clone())
//             .weighted_cyclic(input_size, output_size, memory_size, Op::linear())
//             .build();
//         GraphCodex::<Op<f32>>::from_graph(nodes, &store)
//     }
// }

// impl<C> Codex<GraphChromosome<C>, Graph<C>> for GraphCodex<C>
// where
//     C: NodeCell + Clone + PartialEq + Default + 'static,
// {
//     fn encode(&self) -> Genotype<GraphChromosome<C>> {
//         let store = self.store.read().unwrap();

//         if let Some(graph) = &self.graph {
//             let nodes = graph
//                 .iter()
//                 .map(|node| {
//                     let temp_node = store.new_instance((node.index(), node.node_type()));

//                     if temp_node.value().arity() == node.value().arity() {
//                         return node.with_allele(temp_node.allele());
//                     }

//                     node.clone()
//                 })
//                 .collect::<Vec<GraphNode<C>>>();

//             return Genotype {
//                 chromosomes: vec![GraphChromosome::new(nodes, Arc::clone(&self.store))],
//             };
//         }

//         panic!("Graph not initialized.");
//     }

//     fn decode(&self, genotype: &Genotype<GraphChromosome<C>>) -> Graph<C> {
//         Graph::new(
//             genotype
//                 .iter()
//                 .next()
//                 .unwrap()
//                 .iter()
//                 .cloned()
//                 .collect::<Vec<GraphNode<C>>>(),
//         )
//     }
// }
