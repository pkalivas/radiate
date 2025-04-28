use super::{Direction, Graph, GraphNode};
use crate::{Arity, NodeType, node::Node};
use radiate_core::{Valid, random_provider};
use std::{collections::HashSet, fmt::Debug, ops::Deref};

/// Represents a reversible change to the graph
#[derive(Debug, Clone)]
pub enum MutationStep {
    AddNode(usize),
    AddEdge(usize, usize),
    RemoveEdge(usize, usize),
    DirectionChange {
        index: usize,
        previous_direction: Direction,
    },
}

#[derive(Clone)]
pub enum ReplayStep<T> {
    AddNode(usize, Option<GraphNode<T>>),
    AddEdge(usize, usize),
    RemoveEdge(usize, usize),
    DirectionChange(usize, Direction),
}

pub enum TransactionResult<T> {
    Valid(Vec<MutationStep>),
    Invalid(Vec<MutationStep>, Vec<ReplayStep<T>>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InsertStep {
    Detach(usize, usize),
    Connect(usize, usize),
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

    pub fn commit(self) -> TransactionResult<T> {
        self.commit_with::<fn(&Graph<T>) -> bool>(None)
    }

    pub fn commit_with<F: Fn(&Graph<T>) -> bool>(
        mut self,
        validator: Option<F>,
    ) -> TransactionResult<T> {
        self.set_cycles();
        let result_steps = self.steps.iter().map(|step| (*step).clone()).collect();

        if let Some(validator) = validator {
            return if validator(self.graph) && self.is_valid() {
                TransactionResult::Valid(result_steps)
            } else {
                let replay_steps = self.rollback();
                TransactionResult::Invalid(result_steps, replay_steps)
            };
        }

        if self.is_valid() {
            TransactionResult::Valid(result_steps)
        } else {
            let replay_steps = self.rollback();
            TransactionResult::Invalid(result_steps, replay_steps)
        }
    }

    pub fn add_node(&mut self, node: impl Into<GraphNode<T>>) -> usize {
        let index = self.graph.len();
        self.steps.push(MutationStep::AddNode(index));
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
        if let Some(node) = self.graph.get_mut(index) {
            if node.direction() == direction {
                return;
            }

            self.steps.push(MutationStep::DirectionChange {
                index,
                previous_direction: node.direction(),
            });
            node.set_direction(direction);
        }
    }

    pub fn rollback(self) -> Vec<ReplayStep<T>> {
        let mut replay_steps = Vec::new();
        for step in self.steps.into_iter().rev() {
            match step {
                MutationStep::AddNode(_) => {
                    let added_node = self.graph.pop();
                    replay_steps.push(ReplayStep::AddNode(self.graph.len(), added_node));
                }
                MutationStep::AddEdge(from, to) => {
                    self.graph.detach(from, to);
                    replay_steps.push(ReplayStep::AddEdge(from, to));
                }
                MutationStep::RemoveEdge(from, to) => {
                    self.graph.attach(from, to);
                    replay_steps.push(ReplayStep::RemoveEdge(from, to));
                }
                MutationStep::DirectionChange {
                    index,
                    previous_direction,
                    ..
                } => {
                    if let Some(node) = self.graph.get_mut(index) {
                        let prev_dir = node.direction();
                        node.set_direction(previous_direction);
                        replay_steps.push(ReplayStep::DirectionChange(index, prev_dir));
                    }
                }
            }
        }

        replay_steps.reverse();
        replay_steps
    }

    pub fn replay(&mut self, steps: Vec<ReplayStep<T>>) {
        for step in steps {
            match step {
                ReplayStep::AddNode(_, node) => {
                    if let Some(node) = node {
                        self.add_node(node);
                    }
                }
                ReplayStep::AddEdge(from, to) => {
                    self.attach(from, to);
                }
                ReplayStep::RemoveEdge(from, to) => {
                    self.detach(from, to);
                }
                ReplayStep::DirectionChange(index, direction) => {
                    self.change_direction(index, direction);
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

    pub fn get_insertion_steps(
        &self,
        source_idx: usize,
        target_idx: usize,
        new_node_idx: usize,
    ) -> Vec<InsertStep> {
        let target_node = self.graph.get(target_idx).unwrap();
        let source_node = self.graph.get(source_idx).unwrap();
        let new_node = self.graph.get(new_node_idx).unwrap();

        let mut steps = Vec::new();

        let source_is_edge = source_node.node_type() == NodeType::Edge;
        let target_is_edge = target_node.node_type() == NodeType::Edge;
        let new_node_arity = new_node.arity();

        if new_node_arity == Arity::Zero && !target_node.is_locked() {
            steps.push(InsertStep::Connect(new_node_idx, target_idx));
            return steps;
        }

        if source_is_edge {
            let source_outgoing_idxes = source_node.outgoing().iter().collect::<Vec<&usize>>();
            let source_outgoing = *random_provider::choose(&source_outgoing_idxes);

            if source_outgoing == &new_node_idx {
                steps.push(InsertStep::Connect(source_idx, new_node_idx));
            } else {
                steps.push(InsertStep::Connect(source_idx, new_node_idx));
                steps.push(InsertStep::Connect(new_node_idx, *source_outgoing));
                steps.push(InsertStep::Detach(source_idx, *source_outgoing));
            }
        } else if target_is_edge || target_node.is_locked() {
            let target_incoming_idxes = target_node.incoming().iter().collect::<Vec<&usize>>();
            let target_incoming = *random_provider::choose(&target_incoming_idxes);

            if target_incoming == &new_node_idx {
                steps.push(InsertStep::Connect(*target_incoming, new_node_idx));
            } else {
                steps.push(InsertStep::Connect(*target_incoming, new_node_idx));
                steps.push(InsertStep::Connect(new_node_idx, target_idx));
                steps.push(InsertStep::Detach(*target_incoming, target_idx));
            }
        } else {
            steps.push(InsertStep::Connect(source_idx, new_node_idx));
            steps.push(InsertStep::Connect(new_node_idx, target_idx));
        }

        steps
    }

    /// The below functions are used to get random nodes from the graph. These are useful for
    /// creating connections between nodes. Neither of these functions will return an edge node.
    /// This is because edge nodes are not valid source or target nodes for connections as they
    /// only allow one incoming and one outgoing connection, thus they can't be used to create
    /// new connections. Instead, edge nodes are used to represent the weights of the connections
    ///
    /// Get a random node that can be used as a source node for a connection.
    /// A source node can be either an input or a vertex node.
    #[inline]
    pub fn random_source_node(&self) -> Option<&GraphNode<T>> {
        self.random_node_of_type(vec![NodeType::Input, NodeType::Vertex, NodeType::Edge])
    }
    /// Get a random node that can be used as a target node for a connection.
    /// A target node can be either an output or a vertex node.
    #[inline]
    pub fn random_target_node(&self) -> Option<&GraphNode<T>> {
        self.random_node_of_type(vec![NodeType::Output, NodeType::Vertex, NodeType::Edge])
    }
    /// Helper functions to get a random node of the specified type. If no nodes of the specified
    /// type are found, the function will try to get a random node of a different type.
    /// If no nodes are found, the function will panic.
    #[inline]
    fn random_node_of_type(&self, node_types: Vec<NodeType>) -> Option<&GraphNode<T>> {
        if node_types.is_empty() {
            return None;
        }

        let gene_node_type_index = random_provider::range(0..node_types.len());
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

        let index = random_provider::range(0..genes.len());
        genes.get(index).copied()
    }
}

impl<T> Deref for GraphTransaction<'_, T> {
    type Target = Graph<T>;

    fn deref(&self) -> &Self::Target {
        self.graph
    }
}

// /// Check if connecting the source node to the target node would create a cycle.
// ///
// /// # Arguments
// /// - source: The index of the source node.
// /// - target: The index of the target node.
// ///
// #[inline]
// pub fn would_create_cycle(&self, source: usize, target: usize) -> bool {
//     let mut seen = HashSet::new();
//     let mut visited = self
//         .get(target)
//         .map(|node| node.outgoing().iter().collect())
//         .unwrap_or(Vec::new());

//     while !visited.is_empty() {
//         let node_index = visited.pop().unwrap();

//         seen.insert(*node_index);

//         if *node_index == source {
//             return true;
//         }

//         let node_edges = self
//             .get(*node_index)
//             .map(|node| {
//                 node.outgoing()
//                     .iter()
//                     .filter(|edge_index| !seen.contains(edge_index))
//                     .collect()
//             })
//             .unwrap_or(Vec::new());

//         for edge_index in node_edges {
//             visited.push(edge_index);
//         }
//     }

//     false
// }
