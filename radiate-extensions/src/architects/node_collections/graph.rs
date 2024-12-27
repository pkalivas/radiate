use radiate::Valid;

use super::GraphIterator;
use crate::node::Node;
use crate::{
    node_collections, schema::collection_type::CollectionType, Direction, NodeCollection,
    NodeRepairs, NodeType, OpNodeFactory,
};

#[derive(Clone, PartialEq, Default)]
pub struct Graph<T>
where
    T: Clone + PartialEq,
{
    pub nodes: Vec<Node<T>>,
}

impl<T> Graph<T>
where
    T: Clone + PartialEq + Default,
{
    pub fn topological_iter(&self) -> impl Iterator<Item = &Node<T>> {
        GraphIterator::new(self)
    }

    pub fn set_cycles(mut self, indecies: Vec<usize>) -> Graph<T> {
        if indecies.is_empty() {
            let all_indices = self
                .get_nodes()
                .iter()
                .map(|node| node.index)
                .collect::<Vec<usize>>();

            return self.set_cycles(all_indices);
        }

        for idx in indecies {
            let node_cycles = node_collections::get_cycles(self.get_nodes(), idx);

            if node_cycles.is_empty() {
                let node = self.get_mut(idx);
                node.direction = Direction::Forward;
            } else {
                for cycle_idx in node_cycles {
                    let node = self.get_mut(cycle_idx);
                    node.direction = Direction::Backward;
                }
            }
        }

        self
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

impl<T> NodeCollection<T> for Graph<T>
where
    T: Clone + PartialEq + Default,
{
    fn from_nodes(nodes: Vec<Node<T>>) -> Self {
        Graph { nodes }
    }

    fn get(&self, index: usize) -> &Node<T> {
        self.nodes.get(index).unwrap_or_else(|| {
            panic!(
                "Node index {} out of bounds. Graph has {} nodes.",
                index,
                self.nodes.len()
            )
        })
    }

    fn get_mut(&mut self, index: usize) -> &mut Node<T> {
        let length = self.nodes.len();
        self.nodes.get_mut(index).unwrap_or_else(|| {
            panic!(
                "Node index {} out of bounds. Graph has {} nodes.",
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

impl<T> NodeRepairs<T> for Graph<T>
where
    T: Clone + PartialEq + Default,
{
    fn repair(&mut self, factory: Option<&OpNodeFactory<T>>) -> Self {
        let mut collection = self.clone().set_cycles(Vec::new());

        for node in collection.iter_mut() {
            node.collection_type = Some(CollectionType::Graph);

            if let Some(factory) = factory {
                let temp_node = factory.new_node(node.index, NodeType::Aggregate);

                if node.node_type() == &NodeType::Output && !node.outgoing().is_empty() {
                    node.node_type = NodeType::Aggregate;
                    node.value = temp_node.value.clone();
                } else if node.node_type() == &NodeType::Input && !node.incoming().is_empty() {
                    node.node_type = NodeType::Aggregate;
                    node.value = temp_node.value.clone();
                }
            }
        }

        collection
    }
}

impl<T> Valid for Graph<T>
where
    T: Clone + PartialEq + Default,
{
    fn is_valid(&self) -> bool {
        self.nodes.iter().all(|node| node.is_valid())
    }
}

impl<T> IntoIterator for Graph<T>
where
    T: Clone + PartialEq + Default,
{
    type Item = Node<T>;
    type IntoIter = std::vec::IntoIter<Node<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.nodes.into_iter()
    }
}

impl<T> std::fmt::Debug for Graph<T>
where
    T: Clone + PartialEq + Default + std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Graph {{\n")?;
        for node in self.get_nodes() {
            write!(f, "  {:?},\n", node)?;
        }
        write!(f, "}}")
    }
}
