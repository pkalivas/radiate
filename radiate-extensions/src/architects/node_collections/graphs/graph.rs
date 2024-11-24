use radiate::engines::genome::genes::gene::Valid;

use crate::{architects::node_collections::graph_node::GraphNode, node_collection, Direction};

use super::{super::node_collection::NodeCollection, GraphIterator};

pub struct Graph<T>
where
    T: Clone + PartialEq,
{
    pub nodes: Vec<GraphNode<T>>,
}

impl<T> Graph<T>
where
    T: Clone + PartialEq + Default,
{
    pub fn new() -> Self {
        Graph::default()
    }

    pub fn topological_iter(&self) -> impl Iterator<Item = &GraphNode<T>> {
        GraphIterator::new(&self)
    }
}

impl<T> NodeCollection<Graph<T>, T> for Graph<T>
where
    T: Clone + PartialEq + Default,
{
    fn from_nodes(nodes: Vec<GraphNode<T>>) -> Self {
        Graph { nodes }
    }

    fn get(&self, index: usize) -> Option<&GraphNode<T>> {
        self.nodes.get(index)
    }

    fn get_mut(&mut self, index: usize) -> Option<&mut GraphNode<T>> {
        self.nodes.get_mut(index)
    }

    fn get_nodes(&self) -> &[GraphNode<T>] {
        &self.nodes
    }

    fn get_nodes_mut(&mut self) -> &mut [GraphNode<T>] {
        &mut self.nodes
    }

    fn set_cycles(mut self, indecies: Vec<usize>) -> Graph<T> {
        if indecies.len() == 0 {
            let all_indices = self
                .get_nodes()
                .iter()
                .map(|node| node.index)
                .collect::<Vec<usize>>();

            return self.set_cycles(all_indices);
        }

        for idx in indecies {
            let node_cycles = node_collection::get_cycles(self.get_nodes(), idx);

            if node_cycles.len() == 0 {
                let node = self.get_mut(idx).unwrap();
                (*node).direction = Direction::Forward;
            } else {
                for cycle_idx in node_cycles {
                    let node = self.get_mut(cycle_idx).unwrap();
                    (*node).direction = Direction::Backward;
                }
            }
        }

        self
    }

    fn add(&mut self, nodes: Vec<GraphNode<T>>) {
        self.nodes.extend(nodes);
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

impl<T> Clone for Graph<T>
where
    T: Clone + PartialEq + Default,
{
    fn clone(&self) -> Self {
        Graph::from_nodes(
            self.nodes
                .iter()
                .map(|node| node.clone())
                .collect::<Vec<GraphNode<T>>>(),
        )
    }
}

impl<T> Default for Graph<T>
where
    T: Clone + PartialEq + Default,
{
    fn default() -> Self {
        Graph { nodes: Vec::new() }
    }
}

impl<T> IntoIterator for Graph<T>
where
    T: Clone + PartialEq + Default,
{
    type Item = GraphNode<T>;
    type IntoIter = std::vec::IntoIter<GraphNode<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.nodes.into_iter()
    }
}

impl<T> FromIterator<GraphNode<T>> for Graph<T>
where
    T: Clone + PartialEq + Default,
{
    fn from_iter<I: IntoIterator<Item = GraphNode<T>>>(iter: I) -> Self {
        let nodes = iter.into_iter().collect();
        Graph { nodes }
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
