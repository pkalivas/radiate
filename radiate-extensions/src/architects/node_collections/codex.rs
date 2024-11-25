use radiate::engines::codexes::Codex;
use radiate::engines::genome::chromosome::Chromosome;
use radiate::engines::genome::genes::gene::Gene;
use radiate::engines::genome::genotype::Genotype;

use crate::architects::*;
use crate::node::Node;
use crate::operations::op::Ops;

pub struct GraphCodex<T>
where
    T: Clone + PartialEq + Default,
{
    pub input_size: usize,
    pub output_size: usize,
    pub factory: NodeFactory<T>,
    pub nodes: Vec<Node<T>>,
}

impl<T> GraphCodex<T>
where
    T: Clone + PartialEq + Default,
{
    pub fn from_factory(factory: &NodeFactory<T>) -> Self {
        GraphCodex::from_shape(1, 1, factory)
    }

    pub fn from_shape(input_size: usize, output_size: usize, factory: &NodeFactory<T>) -> Self {
        let nodes = Architect::<Graph<T>, T>::new(factory)
            .acyclic(input_size, output_size)
            .iter()
            .cloned()
            .collect::<Vec<Node<T>>>();

        GraphCodex::from_nodes(nodes, factory)
    }

    pub fn from_nodes(nodes: Vec<Node<T>>, factory: &NodeFactory<T>) -> Self {
        GraphCodex {
            input_size: nodes
                .iter()
                .filter(|node| node.node_type == NodeType::Input)
                .count(),
            output_size: nodes
                .iter()
                .filter(|node| node.node_type == NodeType::Output)
                .count(),
            factory: factory.clone(),
            nodes,
        }
    }

    pub fn set_nodes<F>(mut self, node_fn: F) -> Self
    where
        F: Fn(&Architect<Graph<T>, T>, NodeCollectionBuilder<Graph<T>, T>) -> Graph<T>,
    {
        let graph = Architect::<Graph<T>, T>::new(&self.factory)
            .build(|arc, builder| node_fn(arc, builder));

        self.nodes = graph.iter().cloned().collect::<Vec<Node<T>>>();
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

    pub fn set_factory(mut self, factory: &NodeFactory<T>) -> Self {
        self.factory = factory.clone();
        self
    }

    pub fn set_gates(mut self, gates: Vec<Ops<T>>) -> Self {
        self.factory.add_node_values(NodeType::Gate, gates);
        self
    }

    pub fn set_weights(mut self, weights: Vec<Ops<T>>) -> Self {
        self.factory.add_node_values(NodeType::Weight, weights);
        self
    }

    pub fn set_aggregates(mut self, aggregates: Vec<Ops<T>>) -> Self {
        self.factory
            .add_node_values(NodeType::Aggregate, aggregates);
        self
    }

    pub fn set_inputs(mut self, inputs: Vec<Ops<T>>) -> Self {
        self.factory.add_node_values(NodeType::Input, inputs);
        self
    }

    pub fn set_outputs(mut self, outputs: Vec<Ops<T>>) -> Self {
        self.factory.add_node_values(NodeType::Output, outputs);
        self
    }

    pub fn get_factory(&self) -> NodeFactory<T> {
        self.factory.clone()
    }
}

impl GraphCodex<f32> {
    pub fn regression(input_size: usize, output_size: usize) -> Self {
        let factory = NodeFactory::<f32>::regression(input_size);
        let nodes = Architect::<Graph<f32>, f32>::new(&factory)
            .acyclic(input_size, output_size)
            .iter()
            .cloned()
            .collect::<Vec<Node<f32>>>();

        GraphCodex::<f32>::from_nodes(nodes, &factory)
    }
}

impl<T> Codex<Node<T>, Ops<T>, Graph<T>> for GraphCodex<T>
where
    T: Clone + PartialEq + Default,
{
    fn encode(&self) -> Genotype<Node<T>, Ops<T>> {
        Genotype {
            chromosomes: vec![Chromosome::from_genes(
                self.nodes
                    .iter()
                    .map(|node| {
                        let temp_node = self.factory.new_node(node.index, node.node_type);

                        if temp_node.value.arity() == node.value.arity() {
                            return node.from_allele(temp_node.allele());
                        }

                        node.clone()
                    })
                    .collect::<Vec<Node<T>>>(),
            )],
        }
    }

    fn decode(&self, genotype: &Genotype<Node<T>, Ops<T>>) -> Graph<T> {
        Graph::from_nodes(
            genotype
                .iter()
                .next()
                .unwrap()
                .iter()
                .cloned()
                .collect::<Vec<Node<T>>>(),
        )
    }
}

pub struct TreeCodex<'a, T>
where
    T: Clone + PartialEq + Default,
{
    pub factory: &'a NodeFactory<T>,
    pub nodes: Vec<Node<T>>,
}

impl<'a, T> TreeCodex<'a, T>
where
    T: Clone + PartialEq + Default,
{
    pub fn new(depth: usize, factory: &'a NodeFactory<T>) -> Self {
        let nodes = Architect::<Tree<T>, T>::new(factory)
            .tree(depth)
            .iter()
            .cloned()
            .collect::<Vec<Node<T>>>();

        TreeCodex { factory, nodes }
    }
}

impl<'a, T> Codex<Node<T>, Ops<T>, Tree<T>> for TreeCodex<'a, T>
where
    T: Clone + PartialEq + Default,
{
    fn encode(&self) -> Genotype<Node<T>, Ops<T>> {
        Genotype {
            chromosomes: vec![Chromosome::from_genes(
                self.nodes
                    .iter()
                    .map(|node| {
                        let temp_node = self.factory.new_node(node.index, node.node_type);

                        if temp_node.value.arity() == node.value.arity() {
                            return node.from_allele(temp_node.allele());
                        }

                        node.clone()
                    })
                    .collect::<Vec<Node<T>>>(),
            )],
        }
    }

    fn decode(&self, genotype: &Genotype<Node<T>, Ops<T>>) -> Tree<T> {
        Tree::from_nodes(
            genotype
                .iter()
                .next()
                .unwrap()
                .iter()
                .cloned()
                .collect::<Vec<Node<T>>>(),
        )
    }
}
