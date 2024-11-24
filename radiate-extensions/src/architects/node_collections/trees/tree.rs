use radiate::Valid;

use crate::{Node, NodeCollection};

#[derive(Clone, PartialEq, Default)]
pub struct Tree<T>
where
    T: Clone + PartialEq + Default,
{
    nodes: Vec<Node<T>>,
}

impl<T> Tree<T>
where
    T: Clone + PartialEq + Default,
{
    pub fn new() -> Self {
        Tree::default()
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

impl<T> NodeCollection<Tree<T>, T> for Tree<T>
where
    T: Clone + PartialEq + Default,
{
    fn from_nodes(nodes: Vec<Node<T>>) -> Self {
        Tree { nodes }
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

    fn set_cycles(self, _: Vec<usize>) -> Tree<T> {
       self
    }

    fn add(&mut self, nodes: Vec<Node<T>>) {
        self.nodes.extend(nodes);
    }
}