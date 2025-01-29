use crate::{node::Node, Arity, NodeType};

use super::{Direction, Graph, GraphNode};
use radiate::{random_provider, Valid};
use std::{
    collections::HashSet,
    ops::{Deref, Index},
};

/// Represents a reversible change to the graph
#[derive(Debug)]
enum MutationStep {
    AddNode,
    AddEdge(usize, usize),
    RemoveEdge(usize, usize),
    DirectionChange {
        index: usize,
        previous_direction: Direction,
    },
}

pub enum InsertionType {
    FeedForward,
    Split,
    Invalid,
}

/// Tracks changes and provides rollback capability
pub struct GraphTransaction<'a, T> {
    graph: &'a mut Graph<T>,
    steps: Vec<MutationStep>,
    effects: HashSet<usize>,
}

impl<'a, T> GraphTransaction<'a, T> {
    pub fn new(graph: &'a mut Graph<T>) -> Self {
        GraphTransaction {
            graph,
            steps: Vec::new(),
            effects: HashSet::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.graph.len()
    }

    pub fn add_node(&mut self, node: GraphNode<T>) -> usize {
        let index = self.graph.len();
        self.steps.push(MutationStep::AddNode);
        self.graph.push(node);
        self.effects.insert(index);
        index
    }

    pub fn attach(&mut self, from: usize, to: usize) {
        self.steps.push(MutationStep::AddEdge(from, to));
        self.graph.attach(from, to);
        self.effects.insert(from);
        self.effects.insert(to);
    }

    pub fn detach(&mut self, from: usize, to: usize) {
        self.steps.push(MutationStep::RemoveEdge(from, to));
        self.graph.detach(from, to);
        self.effects.insert(from);
        self.effects.insert(to);
    }

    pub fn change_direction(&mut self, index: usize, direction: Direction) {
        let previous_direction = self.graph[index].direction();
        self.steps.push(MutationStep::DirectionChange {
            index,
            previous_direction,
        });
        self.graph[index].set_direction(direction);
    }

    pub fn rollback(self) {
        // Reverse all changes in reverse order
        for step in self.steps.into_iter().rev() {
            match step {
                MutationStep::AddNode => {
                    self.graph.pop();
                }
                MutationStep::AddEdge(from, to) => {
                    self.graph.detach(from, to);
                }
                MutationStep::RemoveEdge(from, to) => {
                    self.graph.attach(from, to);
                }
                MutationStep::DirectionChange {
                    index,
                    previous_direction,
                    ..
                } => {
                    self.graph[index].set_direction(previous_direction);
                }
            }
        }
    }

    pub fn is_valid(&self) -> bool {
        self.graph.is_valid()
    }

    pub fn set_cycles(&mut self) {
        let effects = self.effects.clone();

        for idx in effects {
            let node_cycles = self.graph.get_cycles(idx);

            if node_cycles.is_empty() {
                self.change_direction(idx, Direction::Forward);
            } else {
                for cycle_idx in node_cycles {
                    self.change_direction(cycle_idx, Direction::Backward);
                }
            }
        }
    }

    pub fn repair(&mut self, index: usize, allow_recurrent: bool)
    where
        T: Clone,
    {
        let arity = self.graph.get(index).arity();

        match arity {
            Arity::Exact(_) => {
                let incoming_count = self.graph.get(index).incoming().len();
                for _ in 0..*arity - incoming_count {
                    if random_provider::random::<f32>() < 0.05 {
                        let input_node = self
                            .graph
                            .iter()
                            .filter(|node| node.arity() == Arity::Zero)
                            .collect::<Vec<&GraphNode<T>>>();

                        let random_input = random_provider::choose(&input_node).value();
                        let input_index = self.add_node(GraphNode::with_arity(
                            self.len(),
                            NodeType::Input,
                            (*random_input).clone(),
                            Arity::Zero,
                        ));

                        self.attach(input_index, index);
                    } else {
                        let other_source_node = self.random_source_node();
                        let insert_type = self.get_insertion_type(
                            other_source_node.index(),
                            index,
                            allow_recurrent,
                        );

                        match insert_type {
                            InsertionType::Invalid => {}
                            _ => {
                                self.attach(other_source_node.index(), index);
                            }
                        }
                    }
                }
            }
            _ => {}
        }

        self.set_cycles();
    }

    pub fn get_insertion_type(
        &self,
        source: usize,
        target: usize,
        allow_recurrent: bool,
    ) -> InsertionType {
        let target_node = self.graph.get(target);

        let same_node = source == target;

        if allow_recurrent {
            if target_node.is_locked() || same_node {
                return InsertionType::Split;
            }

            return InsertionType::FeedForward;
        }

        let would_create_cycle = self.would_create_cycle(source, target);

        if same_node || would_create_cycle {
            return InsertionType::Invalid;
        }

        if target_node.is_locked() {
            return InsertionType::Split;
        }

        InsertionType::FeedForward
    }

    /// Check if connecting the source node to the target node would create a cycle.
    ///
    /// # Arguments
    /// - source: The index of the source node.
    /// - target: The index of the target node.
    ///
    #[inline]
    pub fn would_create_cycle(&self, source: usize, target: usize) -> bool {
        let mut seen = HashSet::new();
        let mut visited = self.get(target).outgoing().iter().collect::<Vec<&usize>>();

        while !visited.is_empty() {
            let node_index = visited.pop().unwrap();

            seen.insert(*node_index);

            if *node_index == source {
                return true;
            }

            for edge_index in self
                .get(*node_index)
                .outgoing()
                .iter()
                .filter(|edge_index| !seen.contains(edge_index))
            {
                visited.push(edge_index);
            }
        }

        false
    }
    /// The below functinos are used to get random nodes from the graph. These are useful for
    /// creating connections between nodes. Neither of these functions will return an edge node.
    /// This is because edge nodes are not valid source or target nodes for connections as they
    /// they only allow one incoming and one outgoing connection, thus they can't be used to create
    /// new connections. Instread, edge nodes are used to represent the weights of the connections
    ///
    /// Get a random node that can be used as a source node for a connection.
    /// A source node can be either an input or a vertex node.
    #[inline]
    pub fn random_source_node(&self) -> &GraphNode<T> {
        self.random_node_of_type(vec![NodeType::Input, NodeType::Vertex, NodeType::Edge])
    }
    /// Get a random node that can be used as a target node for a connection.
    /// A target node can be either an output or a vertex node.
    #[inline]
    pub fn random_target_node(&self) -> &GraphNode<T> {
        self.random_node_of_type(vec![NodeType::Output, NodeType::Vertex, NodeType::Edge])
    }
    /// Helper functions to get a random node of the specified type. If no nodes of the specified
    /// type are found, the function will try to get a random node of a different type.
    /// If no nodes are found, the function will panic.
    #[inline]
    fn random_node_of_type(&self, node_types: Vec<NodeType>) -> &GraphNode<T> {
        if node_types.is_empty() {
            panic!("At least one node type must be specified.");
        }

        let gene_node_type_index = random_provider::random::<usize>() % node_types.len();
        let gene_node_type = node_types.get(gene_node_type_index).unwrap();

        let genes = match gene_node_type {
            NodeType::Input => self
                .iter()
                .filter(|node| node.node_type() == NodeType::Input)
                .collect::<Vec<&GraphNode<T>>>(),
            NodeType::Output => self
                .iter()
                .filter(|node| node.node_type() == NodeType::Output)
                .collect::<Vec<&GraphNode<T>>>(),
            NodeType::Vertex => self
                .iter()
                .filter(|node| node.node_type() == NodeType::Vertex)
                .collect::<Vec<&GraphNode<T>>>(),
            NodeType::Edge => self
                .iter()
                .filter(|node| node.node_type() == NodeType::Edge)
                .collect::<Vec<&GraphNode<T>>>(),
            _ => vec![],
        };

        if genes.is_empty() {
            return self.random_node_of_type(
                node_types
                    .iter()
                    .filter(|nt| *nt != gene_node_type)
                    .cloned()
                    .collect(),
            );
        }

        let index = random_provider::random::<usize>() % genes.len();
        genes.get(index).unwrap()
    }
}

impl<'a, T> Index<usize> for GraphTransaction<'a, T> {
    type Output = GraphNode<T>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.graph[index]
    }
}

impl<'a, T> Deref for GraphTransaction<'a, T> {
    type Target = Graph<T>;

    fn deref(&self) -> &Self::Target {
        self.graph
    }
}
