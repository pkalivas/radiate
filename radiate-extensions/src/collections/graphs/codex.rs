use crate::collections::graphs::architect::GraphArchitect;
use crate::collections::graphs::builder::GraphBuilder;
use crate::collections::{Graph, GraphNode, NodeChromosome, NodeFactory, NodeType};
use crate::ops::Operation;
use crate::Factory;
use radiate::{Chromosome, Codex, Gene, Genotype};
use std::cell::RefCell;
use std::rc::Rc;

pub struct GraphCodex<T>
where
    T: Clone + PartialEq + Default,
{
    factory: Rc<RefCell<NodeFactory<T>>>,
    graph: Option<Graph<T>>,
}

impl<T> GraphCodex<T>
where
    T: Clone + PartialEq + Default,
{
    pub fn from_factory(factory: &NodeFactory<T>) -> Self {
        GraphCodex::from_shape(1, 1, factory)
    }

    pub fn from_shape(input_size: usize, output_size: usize, factory: &NodeFactory<T>) -> Self {
        let nodes = GraphBuilder::<T>::new(factory).acyclic(input_size, output_size);

        GraphCodex::from_graph(nodes, factory)
    }

    pub fn from_graph(graph: Graph<T>, factory: &NodeFactory<T>) -> Self {
        GraphCodex {
            factory: Rc::new(RefCell::new(factory.clone())),
            graph: Some(graph),
        }
    }

    pub fn set_nodes<F>(mut self, node_fn: F) -> Self
    where
        F: Fn(&GraphBuilder<T>, GraphArchitect<T>) -> Graph<T>,
    {
        let graph = node_fn(
            &GraphBuilder::new(&self.factory.borrow()),
            GraphArchitect::new(),
        );

        self.graph = Some(graph);
        self
    }

    pub fn set_factory(mut self, factory: &NodeFactory<T>) -> Self {
        self.factory = Rc::new(RefCell::new(factory.clone()));
        self
    }

    pub fn set_vertices(self, vertices: Vec<Operation<T>>) -> Self {
        self.set_values(NodeType::Vertex, vertices);
        self
    }

    pub fn set_edges(self, edges: Vec<Operation<T>>) -> Self {
        self.set_values(NodeType::Edge, edges);
        self
    }

    pub fn set_inputs(self, inputs: Vec<Operation<T>>) -> Self {
        self.set_values(NodeType::Input, inputs);
        self
    }

    pub fn set_outputs(self, outputs: Vec<Operation<T>>) -> Self {
        self.set_values(NodeType::Output, outputs);
        self
    }

    pub fn set_values(&self, node_type: NodeType, values: Vec<Operation<T>>) {
        let mut factory = self.factory.borrow_mut();
        factory.add_node_values(node_type, values);
    }
}

impl GraphCodex<f32> {
    pub fn regression(input_size: usize, output_size: usize) -> Self {
        let factory = NodeFactory::<f32>::regression(input_size);
        let nodes = GraphBuilder::<f32>::regression(input_size).acyclic(input_size, output_size);
        GraphCodex::<f32>::from_graph(nodes, &factory)
    }
}

impl<T> Codex<NodeChromosome<T>, Graph<T>> for GraphCodex<T>
where
    T: Clone + PartialEq + Default + 'static,
{
    fn encode(&self) -> Genotype<NodeChromosome<T>> {
        let reader = self.factory.borrow();

        if let Some(graph) = &self.graph {
            let nodes = graph
                .iter()
                .map(|node| {
                    let temp_node = reader.new_instance((node.index, node.node_type));

                    if temp_node.value.arity() == node.value.arity() {
                        return node.with_allele(temp_node.allele());
                    }

                    node.clone()
                })
                .collect::<Vec<GraphNode<T>>>();

            // let node_chrom = NodeChrom::with_factory(nodes, self.factory.clone());

            return Genotype {
                chromosomes: vec![NodeChromosome::with_factory(nodes, self.factory.clone())],
            };
        }

        panic!("Graph not initialized.");
    }

    fn decode(&self, genotype: &Genotype<NodeChromosome<T>>) -> Graph<T> {
        Graph::new(
            genotype
                .iter()
                .next()
                .unwrap()
                .iter()
                .cloned()
                .collect::<Vec<GraphNode<T>>>(),
        )
    }
}
