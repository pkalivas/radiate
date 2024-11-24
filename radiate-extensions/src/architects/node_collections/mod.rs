use std::collections::HashSet;

use radiate::RandomProvider;

use super::schema::node_types::NodeType;

pub mod codexes;
pub mod factories;
pub mod graphs;
pub mod node_collection;
pub mod nodes;

pub use codexes::*;
pub use factories::*;
pub use graphs::*;
pub use node_collection::*;
pub use nodes::*;

#[inline]
pub fn can_connect<T>(collection: &[GraphNode<T>], source: usize, target: usize, recurrent: bool) -> bool
where
    T: Clone + PartialEq + Default,
{
    let source_node = &collection.get(source).unwrap();
    let target_node = &collection.get(target).unwrap();

    if (source_node.outgoing.len() == 0 || source_node.is_recurrent()) && !recurrent {
        return false;
    }

    let would_create_cycle = recurrent || !would_create_cycle(collection, source, target);
    let nodes_are_weights =
        source_node.node_type == NodeType::Weight || target_node.node_type == NodeType::Weight;

    would_create_cycle && !nodes_are_weights && source != target
}

#[inline]
pub fn would_create_cycle<T>(collection: &[GraphNode<T>], source: usize, target: usize) -> bool
where
    T: Clone + PartialEq + Default,
{
    let mut seen = HashSet::new();
    let mut visited = collection
        .get(target)
        .unwrap()
        .outgoing
        .iter()
        .collect::<Vec<&usize>>();

    while visited.len() != 0 {
        let node_index = visited.pop().unwrap();

        seen.insert(*node_index);

        if *node_index == source {
            return true;
        }

        for edge_index in collection
            .get(*node_index)
            .unwrap()
            .outgoing
            .iter()
            .filter(|edge_index| !seen.contains(edge_index))
        {
            visited.push(edge_index);
        }
    }

    false
}

pub fn is_locked<T>(node: &GraphNode<T>) -> bool
where
    T: Clone + PartialEq + Default,
{
    if node.node_type == NodeType::Aggregate || node.node_type == NodeType::Output {
        return false;
    }

    match node.arity() {
        Some(arity) => return node.incoming.len() == *arity as usize,
        None => panic!("Node arity must be set before checking if it is locked."),
    }
}

#[inline]
pub fn random_source_node<T>(collection: &[GraphNode<T>]) -> &GraphNode<T>
where
    T: Clone + PartialEq + Default,
{
    random_node_of_type(
        collection,
        vec![
            NodeType::Input,
            NodeType::Gate,
            NodeType::Aggregate,
            NodeType::Link,
        ],
    )
}

#[inline]
pub fn random_target_node<T>(collection: &[GraphNode<T>]) -> &GraphNode<T>
where
    T: Clone + PartialEq + Default,
{
    random_node_of_type(collection, vec![NodeType::Output, NodeType::Aggregate])
}

#[inline]
fn random_node_of_type<T>(collection: &[GraphNode<T>], node_types: Vec<NodeType>) -> &GraphNode<T>
where
    T: Clone + PartialEq + Default,
{
    if node_types.len() == 0 {
        panic!("At least one node type must be specified.");
    }

    let gene_node_type_index = RandomProvider::random::<usize>() % node_types.len();
    let gene_node_type = node_types.get(gene_node_type_index).unwrap();

    let genes = match gene_node_type {
        NodeType::Input => collection
            .iter()
            .filter(|node| node.node_type == NodeType::Input)
            .collect::<Vec<&GraphNode<T>>>(),
        NodeType::Weight => collection
            .iter()
            .filter(|node| node.node_type == NodeType::Weight)
            .collect::<Vec<&GraphNode<T>>>(),
        NodeType::Gate => collection
            .iter()
            .filter(|node| node.node_type == NodeType::Gate)
            .collect::<Vec<&GraphNode<T>>>(),
        NodeType::Output => collection
            .iter()
            .filter(|node| node.node_type == NodeType::Output)
            .collect::<Vec<&GraphNode<T>>>(),
        NodeType::Link => collection
            .iter()
            .filter(|node| node.node_type == NodeType::Link)
            .collect::<Vec<&GraphNode<T>>>(),
        NodeType::Aggregate => collection
            .iter()
            .filter(|node| node.node_type == NodeType::Aggregate)
            .collect::<Vec<&GraphNode<T>>>(),
    };

    if genes.len() == 0 {
        return random_node_of_type(
            collection,
            node_types
                .iter()
                .filter(|nt| *nt != gene_node_type)
                .cloned()
                .collect(),
        );
    }

    let index = RandomProvider::random::<usize>() % genes.len();
    genes.get(index).unwrap()
}
