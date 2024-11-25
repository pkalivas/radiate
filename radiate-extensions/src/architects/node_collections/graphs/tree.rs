use radiate::Valid;

use crate::{Node, NodeCollection, NodeFactory, NodeRepairs, NodeType};


#[derive(Clone, PartialEq, Default)]
pub struct Tree<T> 
where 
    T: Clone + PartialEq,
{
    pub nodes: Vec<Node<T>>
}

impl<T> NodeCollection<T> for Tree<T>
where
    T: Clone + PartialEq + Default,
{
    fn from_nodes(nodes: Vec<Node<T>>) -> Self {
        Self { nodes }
    }

    fn get(&self, index: usize) -> Option<&Node<T>> {
        self.nodes.get(index)
    }

    fn get_mut(&mut self, index: usize) -> Option<&mut Node<T>> {
        self.nodes.get_mut(index)
    }

    fn get_nodes(&self) -> &[Node<T>] {
        &self.nodes
    }

    fn get_nodes_mut(&mut self) -> &mut [Node<T>] {
        &mut self.nodes
    }
}

impl<T> NodeRepairs<T> for Tree<T>
where
    T: Clone + PartialEq + Default,
{
    fn repair(&mut self, factory: &NodeFactory<T>) -> Self {
        let mut collection = self.clone();

        for node in collection.iter_mut() {
            let arity = node.incoming().len();
            (*node).arity = Some(arity as u8);

            let temp_node = factory.new_node(node.index, NodeType::Aggregate);

            if node.node_type() == &NodeType::Output && node.outgoing().len() > 0 {
                node.node_type = NodeType::Aggregate;
                node.value = temp_node.value.clone();
            } else if node.node_type() == &NodeType::Input && node.incoming().len() > 0 {
                node.node_type = NodeType::Aggregate;
                node.value = temp_node.value.clone();
            }
        }

        collection
    }
}

impl<T> Valid for Tree<T>
where
    T: Clone + PartialEq + Default,
{
    fn is_valid(&self) -> bool {
        self.nodes.iter().all(|node| node.is_valid())
    }
}