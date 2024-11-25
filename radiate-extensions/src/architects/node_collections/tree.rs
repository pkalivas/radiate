use radiate::Valid;

use crate::{
    node_collection, schema::collection_type::CollectionType, Node, NodeCollection, NodeFactory,
    NodeRepairs,
};

use super::BreadthFirstIterator;

#[derive(Clone, PartialEq, Default)]
pub struct Tree<T>
where
    T: Clone + PartialEq,
{
    pub nodes: Vec<Node<T>>,
}

impl<T> Tree<T>
where
    T: Clone + PartialEq + Default,
{
    pub fn new(nodes: Vec<Node<T>>) -> Self {
        Tree { nodes }
    }

    pub fn sub_tree(&self, index: usize) -> Self {
        let nodes = BreadthFirstIterator::new(&self.nodes, index)
            .map(|node| node)
            .collect::<Vec<&Node<T>>>();

        Tree::new(node_collection::reindex(0, nodes.as_slice()))
    }
}

impl<T> NodeCollection<T> for Tree<T>
where
    T: Clone + PartialEq + Default,
{
    fn from_nodes(nodes: Vec<Node<T>>) -> Self {
        Self { nodes }
    }

    fn get(&self, index: usize) -> &Node<T> {
        self.nodes.get(index).unwrap_or_else(|| {
            panic!(
                "Node index {} out of bounds for tree with {} nodes",
                index,
                self.nodes.len()
            )
        })
    }

    fn get_mut(&mut self, index: usize) -> &mut Node<T> {
        let length = self.nodes.len();
        self.nodes.get_mut(index).unwrap_or_else(|| {
            panic!(
                "Node index {} out of bounds for tree with {} nodes",
                index, length
            )
        })
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
    fn repair(&mut self, _: Option<&NodeFactory<T>>) -> Self {
        let mut collection = self.clone();

        for node in collection.iter_mut() {
            let arity = node.outgoing().len();
            (*node).arity = Some(arity as u8);
            (*node).collection_type = Some(CollectionType::Tree);
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

impl<T> std::fmt::Debug for Tree<T>
where
    T: Clone + PartialEq + Default + std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Tree {{\n")?;
        for node in self.get_nodes() {
            write!(f, "  {:?},\n", node)?;
        }
        write!(f, "}}")
    }
}
