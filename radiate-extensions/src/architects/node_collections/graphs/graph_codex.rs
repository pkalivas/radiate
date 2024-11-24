use radiate::engines::codexes::Codex;
use radiate::engines::genome::chromosome::Chromosome;
use radiate::engines::genome::genes::gene::Gene;
use radiate::engines::genome::genotype::Genotype;

use crate::architects::*;
use crate::operations::op::Ops;

pub struct GraphCodex<'a, T>
where
    T: Clone + PartialEq + Default,
{
    pub input_size: usize,
    pub output_size: usize,
    pub factory: &'a NodeFactory<T>,
    pub nodes: Vec<GraphNode<T>>,
}

impl<'a, T> GraphCodex<'a, T>
where
    T: Clone + PartialEq + Default,
{
    pub fn from_factory(factory: &'a NodeFactory<T>) -> Self {
        GraphCodex::from_shape(1, 1, factory)
    }

    pub fn from_shape(input_size: usize, output_size: usize, factory: &'a NodeFactory<T>) -> Self {
        let nodes = Architect::<Graph<T>, T>::new(&factory)
            .acyclic(input_size, output_size)
            .iter()
            .map(|node| node.clone())
            .collect::<Vec<GraphNode<T>>>();

        GraphCodex::from_nodes(nodes, factory)
    }

    pub fn from_nodes(nodes: Vec<GraphNode<T>>, factory: &'a NodeFactory<T>) -> Self {
        GraphCodex {
            input_size: nodes
                .iter()
                .filter(|node| node.node_type == NodeType::Input)
                .count(),
            output_size: nodes
                .iter()
                .filter(|node| node.node_type == NodeType::Output)
                .count(),
            factory,
            nodes,
        }
    }

    pub fn set_nodes<F>(mut self, node_fn: F) -> Self
    where
        F: Fn(&Architect<Graph<T>, T>, NodeCollectionBuilder<Graph<T>, T>) -> Graph<T>,
    {
        let graph = Architect::<Graph<T>, T>::new(&self.factory)
            .build(|arc, builder| node_fn(arc, builder));

        self.nodes = graph
            .iter()
            .map(|node| node.clone())
            .collect::<Vec<GraphNode<T>>>();
        self.input_size = graph
            .iter()
            .filter(|node| node.node_type == NodeType::Input)
            .count();
        self.output_size = graph
            .iter()
            .filter(|node| node.node_type == NodeType::Output)
            .count();
        self
    }

    pub fn input(&self, size: usize) -> Graph<T> {
        self.new_collection(NodeType::Input, size)
    }

    pub fn output(&self, size: usize) -> Graph<T> {
        self.new_collection(NodeType::Output, size)
    }

    pub fn gate(&self, size: usize) -> Graph<T> {
        self.new_collection(NodeType::Gate, size)
    }

    pub fn aggregate(&self, size: usize) -> Graph<T> {
        self.new_collection(NodeType::Aggregate, size)
    }

    pub fn weight(&self, size: usize) -> Graph<T> {
        self.new_collection(NodeType::Weight, size)
    }

    fn new_collection(&self, node_type: NodeType, size: usize) -> Graph<T> {
        let nodes = self.new_nodes(node_type, size);
        Graph::from_nodes(nodes)
    }

    fn new_nodes(&self, node_type: NodeType, size: usize) -> Vec<GraphNode<T>> {
        (0..size)
            .map(|i| self.factory.new_node(i, node_type))
            .collect::<Vec<GraphNode<T>>>()
    }
}

impl<'a, T> Codex<GraphNode<T>, Ops<T>, Graph<T>> for GraphCodex<'a, T>
where
    T: Clone + PartialEq + Default,
{
    fn encode(&self) -> Genotype<GraphNode<T>, Ops<T>> {
        Genotype {
            chromosomes: vec![Chromosome::from_genes(
                self.nodes
                    .iter()
                    .map(|node| {
                        let temp_node = self.factory.new_node(node.index, node.node_type);

                        if temp_node.value.arity() == node.value.arity() {
                            node.from_allele(&temp_node.allele());
                        }

                        node.clone()
                    })
                    .collect::<Vec<GraphNode<T>>>(),
            )],
        }
    }

    fn decode(&self, genotype: &Genotype<GraphNode<T>, Ops<T>>) -> Graph<T> {
        Graph::from_nodes(
            genotype
                .iter()
                .next()
                .unwrap()
                .iter()
                .map(|node| node.clone())
                .collect::<Vec<GraphNode<T>>>(),
        )
    }
}
