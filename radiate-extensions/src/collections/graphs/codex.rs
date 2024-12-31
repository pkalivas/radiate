use crate::collections::graphs::architect::GraphArchitect;
use crate::collections::graphs::builder::GraphBuilder;
use crate::collections::{Graph, GraphNode, NodeFactory, NodeType};
use crate::graphs::chromosome::GraphChromosome;
use crate::ops::Op;
use crate::{Factory, NodeCell};
use radiate::{Chromosome, Codex, Gene, Genotype};
use std::cell::RefCell;
use std::rc::Rc;

pub struct GraphCodex<C>
where
    C: Clone + PartialEq + Default + NodeCell,
{
    factory: Rc<RefCell<NodeFactory<C>>>,
    graph: Option<Graph<C>>,
}

impl<C> GraphCodex<C>
where
    C: Clone + PartialEq + Default + NodeCell,
{
    pub fn from_factory(factory: &NodeFactory<C>) -> Self {
        GraphCodex::from_shape(1, 1, factory)
    }

    pub fn from_shape(input_size: usize, output_size: usize, factory: &NodeFactory<C>) -> Self {
        let nodes = GraphBuilder::<C>::new(factory).acyclic(input_size, output_size);

        GraphCodex::from_graph(nodes, factory)
    }

    pub fn from_graph(graph: Graph<C>, factory: &NodeFactory<C>) -> Self {
        GraphCodex {
            factory: Rc::new(RefCell::new(factory.clone())),
            graph: Some(graph),
        }
    }

    pub fn set_nodes<F>(mut self, node_fn: F) -> Self
    where
        F: Fn(&GraphBuilder<C>, GraphArchitect<C>) -> Graph<C>,
    {
        let graph = node_fn(
            &GraphBuilder::new(&self.factory.borrow()),
            GraphArchitect::new(),
        );

        self.graph = Some(graph);
        self
    }

    pub fn with_vertices(self, vertices: Vec<C>) -> Self {
        self.set_values(NodeType::Vertex, vertices);
        self
    }

    pub fn with_edges(self, edges: Vec<C>) -> Self {
        self.set_values(NodeType::Edge, edges);
        self
    }

    pub fn with_inputs(self, inputs: Vec<C>) -> Self {
        self.set_values(NodeType::Input, inputs);
        self
    }

    pub fn with_output(self, outputs: C) -> Self {
        self.set_values(NodeType::Output, vec![outputs]);
        self
    }

    fn set_values(&self, node_type: NodeType, values: Vec<C>) {
        let mut factory = self.factory.borrow_mut();
        factory.add_node_values(node_type, values);
    }
}

impl GraphCodex<Op<f32>> {
    pub fn regression(input_size: usize, output_size: usize) -> Self {
        let factory = NodeFactory::<Op<f32>>::regression(input_size);
        let nodes = GraphBuilder::<Op<f32>>::new(&factory).acyclic(input_size, output_size);
        GraphCodex::<Op<f32>>::from_graph(nodes, &factory)
    }
}

impl<C> Codex<GraphChromosome<C>, Graph<C>> for GraphCodex<C>
where
    C: NodeCell + Clone + PartialEq + Default + 'static,
{
    fn encode(&self) -> Genotype<GraphChromosome<C>> {
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
                .collect::<Vec<GraphNode<C>>>();

            return Genotype {
                chromosomes: vec![GraphChromosome::new(nodes, self.factory.clone())],
            };
        }

        panic!("Graph not initialized.");
    }

    fn decode(&self, genotype: &Genotype<GraphChromosome<C>>) -> Graph<C> {
        Graph::new(
            genotype
                .iter()
                .next()
                .unwrap()
                .iter()
                .cloned()
                .collect::<Vec<GraphNode<C>>>(),
        )
    }
}
