use super::{Direction, Graph, GraphNode};
use crate::{node::Node, Arity, NodeType};
use radiate::{random_provider, Valid};
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
        let result_steps = self.steps.iter().map(|step| (*step).clone()).collect();

        if self.is_valid() {
            TransactionResult::Valid(result_steps)
        } else {
            let replay_steps = self.rollback();
            TransactionResult::Invalid(result_steps, replay_steps)
        }
    }

    pub fn len(&self) -> usize {
        self.graph.len()
    }

    pub fn add_node(&mut self, node: GraphNode<T>) -> usize {
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

    pub fn get_insertion_type(
        &self,
        source: usize,
        target: usize,
        new_node: usize,
        allow_recurrent: bool,
    ) -> Vec<InsertStep> {
        let target_node = self.graph.get(target).unwrap();
        let source_node = self.graph.get(source).unwrap();

        let mut steps = Vec::new();

        let source_is_edge = source_node.node_type() == NodeType::Edge;
        let target_is_edge = target_node.node_type() == NodeType::Edge;

        let would_create_cycle = self.would_create_cycle(source, target);

        if source_is_edge {
            let source_outgoing = source_node.outgoing().iter().next().unwrap();
            if source_outgoing == &new_node {
                if allow_recurrent {
                    steps.push(InsertStep::Connect(source, new_node));
                } else {
                    steps.push(InsertStep::Invalid);
                }
            } else {
                if would_create_cycle && !allow_recurrent {
                    steps.push(InsertStep::Invalid);
                } else {
                    steps.push(InsertStep::Connect(source, new_node));
                    steps.push(InsertStep::Connect(new_node, *source_outgoing));
                    steps.push(InsertStep::Detach(source, *source_outgoing));
                }
            }
        } else if target_is_edge || target_node.is_locked() {
            let target_incoming = target_node.incoming().iter().next().unwrap();
            if target_incoming == &new_node {
                if allow_recurrent {
                    steps.push(InsertStep::Connect(*target_incoming, new_node));
                } else {
                    steps.push(InsertStep::Invalid);
                }
            } else {
                if would_create_cycle && !allow_recurrent {
                    steps.push(InsertStep::Invalid);
                } else {
                    steps.push(InsertStep::Connect(*target_incoming, new_node));
                    steps.push(InsertStep::Connect(new_node, target));
                    steps.push(InsertStep::Detach(*target_incoming, target));
                }
            }
        } else {
            if allow_recurrent {
                let souce_arity = source_node.arity();

                match souce_arity {
                    Arity::Any => {
                        steps.push(InsertStep::Connect(source, new_node));
                        steps.push(InsertStep::Connect(new_node, source));
                    }
                    _ => {
                        steps.push(InsertStep::Connect(source, new_node));
                        steps.push(InsertStep::Connect(new_node, target));
                    }
                }
            } else {
                if !would_create_cycle && source != target {
                    if source == new_node || target == new_node {
                        steps.push(InsertStep::Invalid);
                    } else {
                        steps.push(InsertStep::Connect(source, new_node));
                        steps.push(InsertStep::Connect(new_node, target));
                    }
                }
            }
        }

        steps
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
        let mut visited = self
            .get(target)
            .map(|node| node.outgoing().iter().collect())
            .unwrap_or(Vec::new());

        while !visited.is_empty() {
            let node_index = visited.pop().unwrap();

            seen.insert(*node_index);

            if *node_index == source {
                return true;
            }

            let node_edges = self
                .get(*node_index)
                .map(|node| {
                    node.outgoing()
                        .iter()
                        .filter(|edge_index| !seen.contains(edge_index))
                        .collect()
                })
                .unwrap_or(Vec::new());

            for edge_index in node_edges {
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
        genes.get(index).map(|x| *x)
    }
}

impl<'a, T> Deref for GraphTransaction<'a, T> {
    type Target = Graph<T>;

    fn deref(&self) -> &Self::Target {
        self.graph
    }
}
