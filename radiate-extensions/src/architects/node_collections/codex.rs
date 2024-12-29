use crate::architects::*;
use crate::expr::Expr;
use architect::{Architect, TreeArchitect};
use core::panic;
use radiate::engines::codexes::Codex;
use radiate::engines::genome::genes::gene::Gene;
use radiate::engines::genome::genotype::Genotype;
use radiate::{random_provider, Chromosome};
use std::cell::RefCell;
use std::sync::Arc;

// pub struct GraphCodex<T>
// where
//     T: Clone + PartialEq + Default,
// {
//     pub input_size: usize,
//     pub output_size: usize,
//     pub factory: Rc<RefCell<NodeFactory<T>>>,
//     pub nodes: Vec<Node<T>>,
// }

// impl<T> GraphCodex<T>
// where
//     T: Clone + PartialEq + Default,
// {
//     pub fn from_factory(factory: &NodeFactory<T>) -> Self {
//         GraphCodex::from_shape(1, 1, factory)
//     }

//     pub fn from_shape(input_size: usize, output_size: usize, factory: &NodeFactory<T>) -> Self {
//         let nodes = Architect::<Graph<T>, T>::new(factory)
//             .acyclic(input_size, output_size)
//             .iter()
//             .cloned()
//             .collect::<Vec<Node<T>>>();

//         GraphCodex::from_nodes(nodes, factory)
//     }

//     pub fn from_nodes(nodes: Vec<Node<T>>, factory: &NodeFactory<T>) -> Self {
//         GraphCodex {
//             input_size: nodes
//                 .iter()
//                 .filter(|node| node.node_type == NodeType::Input)
//                 .count(),
//             output_size: nodes
//                 .iter()
//                 .filter(|node| node.node_type == NodeType::Output)
//                 .count(),
//             factory: Rc::new(RefCell::new(factory.clone())),
//             nodes,
//         }
//     }

//     pub fn set_nodes<F>(mut self, node_fn: F) -> Self
//     where
//         F: Fn(&Architect<Graph<T>, T>, GraphBuilder<Graph<T>, T>) -> Graph<T>,
//     {
//         let graph = Architect::<Graph<T>, T>::new(&self.factory.borrow())
//             .build(|arc, builder| node_fn(arc, builder));

//         self.nodes = graph.iter().cloned().collect::<Vec<Node<T>>>();
//         self.input_size = graph
//             .iter()
//             .filter(|node| node.node_type == NodeType::Input)
//             .count();
//         self.output_size = graph
//             .iter()
//             .filter(|node| node.node_type == NodeType::Output)
//             .count();
//         self
//     }

//     pub fn set_factory(mut self, factory: &NodeFactory<T>) -> Self {
//         self.factory = Rc::new(RefCell::new(factory.clone()));
//         self
//     }

//     pub fn set_gates(self, gates: Vec<Expr<T>>) -> Self {
//         self.set_values(NodeType::Gate, gates);
//         self
//     }

//     pub fn set_weights(self, weights: Vec<Expr<T>>) -> Self {
//         self.set_values(NodeType::Weight, weights);
//         self
//     }

//     pub fn set_aggregates(self, aggregates: Vec<Expr<T>>) -> Self {
//         self.set_values(NodeType::Aggregate, aggregates);
//         self
//     }

//     pub fn set_inputs(self, inputs: Vec<Expr<T>>) -> Self {
//         self.set_values(NodeType::Input, inputs);
//         self
//     }

//     pub fn set_outputs(self, outputs: Vec<Expr<T>>) -> Self {
//         self.set_values(NodeType::Output, outputs);
//         self
//     }

//     fn set_values(&self, node_type: NodeType, values: Vec<Expr<T>>) {
//         let mut factory = self.factory.borrow_mut();
//         factory.add_node_values(node_type, values);
//     }
// }

// impl GraphCodex<f32> {
//     pub fn regression(input_size: usize, output_size: usize) -> Self {
//         let factory = NodeFactory::<f32>::regression(input_size);
//         let nodes = Architect::<Graph<f32>, f32>::new(&factory)
//             .acyclic(input_size, output_size)
//             .iter()
//             .cloned()
//             .collect::<Vec<Node<f32>>>();

//         GraphCodex::<f32>::from_nodes(nodes, &factory)
//     }
// }

// impl<T> Codex<NodeChromosome<T>, Graph<T>> for GraphCodex<T>
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

//     fn decode(&self, genotype: &Genotype<NodeChromosome<T>>) -> Graph<T> {
//         Graph::from_nodes(
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

pub struct TreeCodex<T: Clone> {
    architect: TreeArchitect<T>,
    constraint: Option<Arc<Box<dyn Fn(&TreeNode<T>) -> bool>>>,
}

impl<T: Clone + Default> TreeCodex<T> {
    pub fn new(depth: usize) -> Self {
        TreeCodex {
            architect: TreeArchitect::new(depth),
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

    pub fn gates(mut self, gates: Vec<Expr<T>>) -> Self {
        self.architect = self.architect.gates(gates);
        self
    }

    pub fn leafs(mut self, leafs: Vec<Expr<T>>) -> Self {
        self.architect = self.architect.leafs(leafs);
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

pub struct GraphCodex<T>
where
    T: Clone + PartialEq + Default,
{
    pub input_size: usize,
    pub output_size: usize,
    pub memory_size: usize,
    pub providers: Arc<Box<Vec<Expr<T>>>>,
    pub internal: Arc<Box<Vec<Expr<T>>>>,
    pub outputs: Arc<Box<Vec<Expr<T>>>>,
    pub nodes: Vec<GraphNode<T>>,
}

impl<T> GraphCodex<T>
where
    T: Clone + PartialEq + Default,
{
    pub fn set_gates(mut self, gates: Vec<Expr<T>>) -> Self {
        self.internal = Arc::new(Box::new(gates));
        self
    }

    pub fn set_inputs(mut self, inputs: Vec<Expr<T>>) -> Self {
        self.providers = Arc::new(Box::new(inputs));

        self
    }

    pub fn set_outputs(mut self, outputs: Vec<Expr<T>>) -> Self {
        self.outputs = Arc::new(Box::new(outputs));
        self
    }
}

impl GraphCodex<f32> {
    pub fn dense(input_size: usize, output_size: usize) -> Self {
        let factory = NodeFactory::<f32>::regression(input_size);
        let graph = GraphBuilder::new(
            &factory.get_inputs(),
            &factory.get_operations(),
            &factory.get_outputs(),
        )
        .acyclic(input_size, output_size);

        GraphCodex {
            input_size,
            output_size,
            memory_size: 0,
            providers: Arc::new(Box::new(factory.get_inputs().clone())),
            internal: Arc::new(Box::new(factory.get_operations().clone())),
            outputs: Arc::new(Box::new(factory.get_outputs().clone())),
            nodes: graph.nodes().to_vec(),
        }
    }
}

impl<T> Codex<NodeChrom<GraphNode<T>>, Graph<T>> for GraphCodex<T>
where
    T: Clone + PartialEq + Default,
{
    fn encode(&self) -> Genotype<NodeChrom<GraphNode<T>>> {
        let nodes = self
            .nodes
            .iter()
            .map(|node| {
                if node.cell.role != Role::Internal {
                    return node.clone();
                }

                let temp = random_provider::choose(self.internal.as_ref().as_ref()).new_instance();
                if temp.arity() == node.cell.value.arity() {
                    return node.with_allele(&temp);
                }

                node.clone()
            })
            .collect::<Vec<GraphNode<T>>>();

        let mut chrome = NodeChrom::new(nodes);
        chrome.set_providers(Arc::clone(&self.providers));
        chrome.set_internals(Arc::clone(&self.internal));
        chrome.set_outputs(Arc::clone(&self.outputs));

        Genotype {
            chromosomes: vec![chrome],
        }
    }

    fn decode(&self, genotype: &Genotype<NodeChrom<GraphNode<T>>>) -> Graph<T> {
        let nodes = genotype
            .iter()
            .next()
            .unwrap()
            .iter()
            .cloned()
            .collect::<Vec<GraphNode<T>>>();

        Graph::new(nodes)
    }
}

// impl<T> Codex<NodeChromosome<T>, Graph<T>> for GraphCodex<T>
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
//
//                 if temp_node.value.arity() == node.value.arity() {
//                     return node.with_allele(temp_node.allele());
//                 }
//
//                 node.clone()
//             })
//             .collect::<Vec<Node<T>>>();
//
//         Genotype {
//             chromosomes: vec![NodeChromosome::with_factory(nodes, self.factory.clone())],
//         }
//     }
//
//     fn decode(&self, genotype: &Genotype<NodeChromosome<T>>) -> Graph<T> {
//         Graph::from_nodes(
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

#[cfg(test)]
mod tests {
    use super::*;
    use radiate::engines::codexes::Codex;

    #[test]
    fn test_tree_codex() {
        let codex = TreeCodex::<f32>::new(3)
            .gates(vec![expr::add(), expr::sub(), expr::mul()])
            .leafs(vec![expr::value(1.0), expr::value(2.0)]);
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
