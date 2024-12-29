use crate::architect::GraphArchitect;
use crate::architects::*;
use crate::node::GraphNode;
use crate::operation::Operation;
use core::panic;
use radiate::engines::codexes::Codex;
use radiate::engines::genome::genes::gene::Gene;
use radiate::engines::genome::genotype::Genotype;
use radiate::Chromosome;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

pub struct GraphCodex<T>
where
    T: Clone + PartialEq + Default,
{
    input_size: usize,
    output_size: usize,
    factory: Rc<RefCell<NodeFactory<T>>>,
    nodes: Vec<GraphNode<T>>,
}

impl<T> GraphCodex<T>
where
    T: Clone + PartialEq + Default,
{
    pub fn from_factory(factory: &NodeFactory<T>) -> Self {
        GraphCodex::from_shape(1, 1, factory)
    }

    pub fn from_shape(input_size: usize, output_size: usize, factory: &NodeFactory<T>) -> Self {
        let nodes = GraphBuilder::<T>::new(factory)
            .acyclic(input_size, output_size)
            .iter()
            .cloned()
            .collect::<Vec<GraphNode<T>>>();

        GraphCodex::from_nodes(nodes, factory)
    }

    pub fn from_nodes(nodes: Vec<GraphNode<T>>, factory: &NodeFactory<T>) -> Self {
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
        F: Fn(&GraphBuilder<T>, GraphArchitect<T>) -> Graph<T>,
    {
        let graph = GraphBuilder::<T>::new(&self.factory.borrow())
            .build(|arc, builder| node_fn(arc, builder));

        self.nodes = graph.iter().cloned().collect::<Vec<GraphNode<T>>>();
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

    // pub fn set_gates(self, gates: Vec<Operation<T>>) -> Self {
    //     self.set_values(NodeType::Gate, gates);
    //     self
    // }

    // pub fn set_weights(self, weights: Vec<Operation<T>>) -> Self {
    //     self.set_values(NodeType::Weight, weights);
    //     self
    // }

    // pub fn set_aggregates(self, aggregates: Vec<Operation<T>>) -> Self {
    //     self.set_values(NodeType::Aggregate, aggregates);
    //     self
    // }

    pub fn set_inputs(self, inputs: Vec<Operation<T>>) -> Self {
        self.set_values(NodeType::Input, inputs);
        self
    }

    pub fn set_outputs(self, outputs: Vec<Operation<T>>) -> Self {
        self.set_values(NodeType::Output, outputs);
        self
    }

    fn set_values(&self, node_type: NodeType, values: Vec<Operation<T>>) {
        let mut factory = self.factory.borrow_mut();
        factory.add_node_values(node_type, values);
    }
}

impl GraphCodex<f32> {
    pub fn regression(input_size: usize, output_size: usize) -> Self {
        let factory = NodeFactory::<f32>::regression(input_size);
        let nodes = GraphBuilder::<f32>::new(&factory)
            .acyclic(input_size, output_size)
            .iter()
            .cloned()
            .collect::<Vec<GraphNode<f32>>>();

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
            .collect::<Vec<GraphNode<T>>>();

        Genotype {
            chromosomes: vec![NodeChromosome::with_factory(nodes, self.factory.clone())],
        }
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

pub struct TreeCodex<T: Clone> {
    architect: TreeBuilder<T>,
    constraint: Option<Arc<Box<dyn Fn(&TreeNode<T>) -> bool>>>,
}

impl<T: Clone + Default> TreeCodex<T> {
    pub fn new(depth: usize) -> Self {
        TreeCodex {
            architect: TreeBuilder::new(depth),
            constraint: None,
        }
    }

    pub fn constraint<F>(mut self, constraint: F) -> Self
    where
        F: Fn(&TreeNode<T>) -> bool + 'static,
    {
        self.constraint = Some(Arc::new(Box::new(constraint)));
        self
    }

    pub fn gates(mut self, gates: Vec<Operation<T>>) -> Self {
        self.architect = self.architect.with_gates(gates);
        self
    }

    pub fn leafs(mut self, leafs: Vec<Operation<T>>) -> Self {
        self.architect = self.architect.with_leafs(leafs);
        self
    }
}

impl<T> Codex<NodeChrom<TreeNode<T>>, Tree<T>> for TreeCodex<T>
where
    T: Clone + PartialEq + Default,
{
    fn encode(&self) -> Genotype<NodeChrom<TreeNode<T>>> {
        let root = self.architect.build().root().take().unwrap().to_owned();

        if let Some(constraint) = &self.constraint {
            if !constraint(&root) {
                panic!("Root node does not meet constraint.");
            }
        }

        Genotype {
            chromosomes: vec![NodeChrom::with_constraint(
                vec![root],
                self.constraint.clone(),
            )],
        }
    }

    fn decode(&self, genotype: &Genotype<NodeChrom<TreeNode<T>>>) -> Tree<T> {
        let nodes = genotype
            .iter()
            .next()
            .unwrap()
            .iter()
            .cloned()
            .collect::<Vec<TreeNode<T>>>()
            .first()
            .unwrap()
            .to_owned();

        Tree::new(nodes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use radiate::engines::codexes::Codex;

    #[test]
    fn test_tree_codex() {
        let codex = TreeCodex::<f32>::new(3)
            .gates(vec![operation::add(), operation::sub(), operation::mul()])
            .leafs(vec![operation::value(1.0), operation::value(2.0)]);
        let genotype = codex.encode();
        let tree = codex.decode(&genotype);

        assert!(tree.root().is_some());
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

//     pub fn set_gates(self, gates: Vec<Expr<T>>) -> Self {
//         self.set_values(NodeType::Gate, gates);
//         self
//     }

//     pub fn set_leafs(self, leafs: Vec<Expr<T>>) -> Self {
//         self.set_values(NodeType::Leaf, leafs);
//         self
//     }

//     fn set_values(&self, node_type: NodeType, values: Vec<Expr<T>>) {
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
