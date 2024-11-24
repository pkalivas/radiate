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
    pub nodes: Vec<Node<T>>,
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
            .collect::<Vec<Node<T>>>();

        GraphCodex::from_nodes(nodes, factory)
    }

    pub fn from_nodes(nodes: Vec<Node<T>>, factory: &'a NodeFactory<T>) -> Self {
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
            .collect::<Vec<Node<T>>>();
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
}

impl<'a, T> Codex<Node<T>, Ops<T>, Graph<T>> for GraphCodex<'a, T>
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
                            node.from_allele(&temp_node.allele());
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
                .map(|node| node.clone())
                .collect::<Vec<Node<T>>>(),
        )
    }
}
