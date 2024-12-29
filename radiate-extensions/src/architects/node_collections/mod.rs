pub mod chromosome;
pub mod codex;
pub mod expr;
pub mod graph;
pub mod iter;
pub mod node;
pub mod node_factory;
pub mod reducers;
pub mod tree;

pub use chromosome::*;
pub use codex::*;
pub use graph::*;
pub use iter::*;
pub use node::*;
pub use node_factory::*;
pub use reducers::*;
pub use tree::*;

use radiate::engines::genome::genes::gene::Valid;

pub trait NodeRepairs<T>
where
    T: Clone,
{
    fn repair(&mut self, factory: Option<&NodeFactory<T>>) -> Self;
}

pub trait NodeCollection<T>: Valid + Default + Clone
where
    T: Clone + PartialEq + Default,
{
    fn from_nodes(nodes: Vec<GraphNode<T>>) -> Self;

    fn get(&self, index: usize) -> &GraphNode<T>;
    fn get_mut(&mut self, index: usize) -> &mut GraphNode<T>;

    fn get_nodes(&self) -> &[GraphNode<T>];
    fn get_nodes_mut(&mut self) -> &mut [GraphNode<T>];

    fn set(&mut self, index: usize, node: GraphNode<T>) -> &mut Self {
        self.get_nodes_mut()[index] = node;
        self
    }

    fn iter(&self) -> std::slice::Iter<GraphNode<T>> {
        self.get_nodes().iter()
    }

    fn iter_mut(&mut self) -> std::slice::IterMut<GraphNode<T>> {
        self.get_nodes_mut().iter_mut()
    }

    fn len(&self) -> usize {
        self.get_nodes().len()
    }

    fn is_empty(&self) -> bool {
        self.get_nodes().is_empty()
    }

    fn attach(&mut self, incoming: usize, outgoing: usize) -> &mut Self {
        self.get_nodes_mut()[incoming]
            .outgoing_mut()
            .insert(outgoing);
        self.get_nodes_mut()[outgoing]
            .incoming_mut()
            .insert(incoming);
        self
    }

    fn detach(&mut self, incoming: usize, outgoing: usize) -> &mut Self {
        self.get_nodes_mut()[incoming]
            .outgoing_mut()
            .remove(&outgoing);
        self.get_nodes_mut()[outgoing]
            .incoming_mut()
            .remove(&incoming);
        self
    }
}
