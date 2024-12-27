use crate::architects::node_collections::nodes::op::Ops;
use crate::architects::*;
use crate::node::Node;
use radiate::engines::codexes::Codex;
use radiate::engines::genome::genes::gene::Gene;
use radiate::engines::genome::genotype::Genotype;
use radiate::Chromosome;
use std::cell::RefCell;
use std::rc::Rc;

pub struct GraphCodex<T>
where
    T: Clone + PartialEq + Default,
{
    pub input_size: usize,
    pub output_size: usize,
    pub factory: Rc<RefCell<OpNodeFactory<T>>>,
    pub nodes: Vec<Node<T>>,
}

impl<T> GraphCodex<T>
where
    T: Clone + PartialEq + Default,
{
    pub fn from_factory(factory: &OpNodeFactory<T>) -> Self {
        GraphCodex::from_shape(1, 1, factory)
    }

    pub fn from_shape(input_size: usize, output_size: usize, factory: &OpNodeFactory<T>) -> Self {
        let nodes = Architect::<Graph<T>, T>::new(factory)
            .acyclic(input_size, output_size)
            .iter()
            .cloned()
            .collect::<Vec<Node<T>>>();

        GraphCodex::from_nodes(nodes, factory)
    }

    pub fn from_nodes(nodes: Vec<Node<T>>, factory: &OpNodeFactory<T>) -> Self {
        GraphCodex {
            input_size: nodes
                .iter()
                .filter(|node| node.node_type == NodeType::Input)
                .count(),
            output_size: nodes
                .iter()
                .filter(|node| node.node_type == NodeType::Output)
                .count(),
            factory: Rc::new(RefCell::new(factory.clone())),
            nodes,
        }
    }

    pub fn set_nodes<F>(mut self, node_fn: F) -> Self
    where
        F: Fn(&Architect<Graph<T>, T>, NodeCollectionBuilder<Graph<T>, T>) -> Graph<T>,
    {
        let graph = Architect::<Graph<T>, T>::new(&self.factory.borrow())
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

    pub fn set_factory(mut self, factory: &OpNodeFactory<T>) -> Self {
        self.factory = Rc::new(RefCell::new(factory.clone()));
        self
    }

    pub fn set_gates(self, gates: Vec<Ops<T>>) -> Self {
        self.set_values(NodeType::Gate, gates);
        self
    }

    pub fn set_weights(self, weights: Vec<Ops<T>>) -> Self {
        self.set_values(NodeType::Weight, weights);
        self
    }

    pub fn set_aggregates(self, aggregates: Vec<Ops<T>>) -> Self {
        self.set_values(NodeType::Aggregate, aggregates);
        self
    }

    pub fn set_inputs(self, inputs: Vec<Ops<T>>) -> Self {
        self.set_values(NodeType::Input, inputs);
        self
    }

    pub fn set_outputs(self, outputs: Vec<Ops<T>>) -> Self {
        self.set_values(NodeType::Output, outputs);
        self
    }

    fn set_values(&self, node_type: NodeType, values: Vec<Ops<T>>) {
        let mut factory = self.factory.borrow_mut();
        factory.add_node_values(node_type, values);
    }
}

impl GraphCodex<f32> {
    pub fn regression(input_size: usize, output_size: usize) -> Self {
        let factory = OpNodeFactory::<f32>::regression(input_size);
        let nodes = Architect::<Graph<f32>, f32>::new(&factory)
            .acyclic(input_size, output_size)
            .iter()
            .cloned()
            .collect::<Vec<Node<f32>>>();

        GraphCodex::<f32>::from_nodes(nodes, &factory)
    }
}

impl<T> Codex<NodeChromosome<T>, Graph<T>> for GraphCodex<T>
where
    T: Clone + PartialEq + Default,
{
    fn encode(&self) -> Genotype<NodeChromosome<T>> {
        let reader = self.factory.borrow();
        let nodes = self
            .nodes
            .iter()
            .map(|node| {
                let temp_node = reader.new_node(node.index, node.node_type);

                if temp_node.value.arity() == node.value.arity() {
                    return node.with_allele(temp_node.allele());
                }

                node.clone()
            })
            .collect::<Vec<Node<T>>>();

        Genotype {
            chromosomes: vec![NodeChromosome::with_factory(nodes, self.factory.clone())],
        }
    }

    fn decode(&self, genotype: &Genotype<NodeChromosome<T>>) -> Graph<T> {
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

// /// A codex for encoding and decoding a tree structure.
// ///
// pub struct TreeCodex<T>
// where
//     T: Clone + PartialEq + Default,
// {
//     pub factory: Rc<RefCell<NodeFactory<T>>>,
//     pub nodes: Vec<Node<T>>,
// }

// impl<T> TreeCodex<T>
// where
//     T: Clone + PartialEq + Default,
// {
//     pub fn new(depth: usize, factory: NodeFactory<T>) -> Self {
//         let nodes = Architect::<Tree<T>, T>::new(&factory)
//             .tree(depth)
//             .iter()
//             .cloned()
//             .collect::<Vec<Node<T>>>();

//         TreeCodex {
//             factory: Rc::new(RefCell::new(factory)),
//             nodes,
//         }
//     }

//     pub fn set_gates(self, gates: Vec<Ops<T>>) -> Self {
//         self.set_values(NodeType::Gate, gates);
//         self
//     }

//     pub fn set_leafs(self, leafs: Vec<Ops<T>>) -> Self {
//         self.set_values(NodeType::Leaf, leafs);
//         self
//     }

//     fn set_values(&self, node_type: NodeType, values: Vec<Ops<T>>) {
//         let mut factory = self.factory.borrow_mut();
//         factory.add_node_values(node_type, values);
//     }
// }

// impl TreeCodex<f32> {
//     pub fn regression(input_size: usize, depth: usize) -> Self {
//         let factory = NodeFactory::<f32>::regression(input_size);
//         let nodes = Architect::<Tree<f32>, f32>::new(&factory)
//             .tree(depth)
//             .iter()
//             .cloned()
//             .collect::<Vec<Node<f32>>>();

//         TreeCodex::<f32> {
//             factory: Rc::new(RefCell::new(factory)),
//             nodes,
//         }
//     }
// }

// impl<T> Codex<NodeChromosome<T>, Tree<T>> for TreeCodex<T>
// where
//     T: Clone + PartialEq + Default,
// {
//     fn encode(&self) -> Genotype<NodeChromosome<T>> {
//         let reader = self.factory.borrow();
//         let nodes = self
//             .nodes
//             .iter()
//             .map(|node| {
//                 let temp_node = reader.new_node(node.index, node.node_type);

//                 if temp_node.value.arity() == node.value.arity() {
//                     return node.with_allele(temp_node.allele());
//                 }

//                 node.clone()
//             })
//             .collect::<Vec<Node<T>>>();

//         Genotype {
//             chromosomes: vec![NodeChromosome::with_factory(nodes, self.factory.clone())],
//         }
//     }

//     fn decode(&self, genotype: &Genotype<NodeChromosome<T>>) -> Tree<T> {
//         Tree::from_nodes(
//             genotype
//                 .iter()
//                 .next()
//                 .unwrap()
//                 .iter()
//                 .cloned()
//                 .collect::<Vec<Node<T>>>(),
//         )
//     }
// }
