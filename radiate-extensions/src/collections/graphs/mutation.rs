use std::collections::HashSet;

use super::{Direction, Graph, GraphNode};
use crate::node::NodeType;
use crate::ops::Arity;
use crate::{CellStore, Factory, GraphMutator, NodeCell};
use radiate::Valid;

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

/// Tracks changes and provides rollback capability
pub struct GraphTransaction<'a, C: NodeCell>
where
    C: Clone + Default + PartialEq + NodeCell,
{
    graph: &'a mut Graph<C>,
    steps: Vec<MutationStep>,
    effects: HashSet<usize>,
}

impl<'a, C: Clone + Default + PartialEq + NodeCell> GraphTransaction<'a, C> {
    pub fn new(graph: &'a mut Graph<C>) -> Self {
        Self {
            graph,
            steps: Vec::new(),
            effects: HashSet::new(),
        }
    }

    pub fn add_node(&mut self, node: GraphNode<C>) -> usize {
        let index = self.graph.nodes.len();
        self.steps.push(MutationStep::AddNode);
        self.graph.nodes.push(node);
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
        let previous_direction = self.graph.nodes[index].direction;
        self.steps.push(MutationStep::DirectionChange {
            index,
            previous_direction,
        });
        self.graph.nodes[index].direction = direction;
    }

    pub fn rollback(self) {
        // Reverse all changes in reverse order
        for step in self.steps.into_iter().rev() {
            match step {
                MutationStep::AddNode => {
                    self.graph.nodes.pop();
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
                    self.graph.nodes[index].direction = previous_direction;
                }
            }
        }
    }

    pub fn is_valid(&self) -> bool {
        self.graph.is_valid()
    }
}

// updated GraphMutator implementation
impl GraphMutator {
    pub fn add_node<C: NodeCell + Clone + Default + PartialEq>(
        &self,
        graph: &mut Graph<C>,
        node_type: &NodeType,
        factory: &CellStore<C>,
        recurrent: bool,
    ) -> bool {
        let mut transaction = GraphTransaction::new(graph);

        if !self.try_add_node(&mut transaction, node_type, factory, recurrent) {
            transaction.rollback();
            return false;
        }

        true
    }

    fn try_add_node<C>(
        &self,
        transaction: &mut GraphTransaction<C>,
        node_type: &NodeType,
        factory: &CellStore<C>,
        is_recurrent: bool,
    ) -> bool
    where
        C: Clone + Default + PartialEq + NodeCell,
    {
        let source_node_index = transaction.graph.random_source_node().index;
        let target_node_index = transaction.graph.random_target_node().index;

        let source_node_type = transaction.graph.nodes[source_node_index].node_type;

        if source_node_type == NodeType::Edge && node_type != &NodeType::Edge {
            if is_recurrent {
                self.try_backward_edge_insertion(
                    transaction,
                    source_node_index,
                    target_node_index,
                    node_type,
                    factory,
                )
            } else {
                self.try_edge_insertion(
                    transaction,
                    source_node_index,
                    target_node_index,
                    node_type,
                    factory,
                )
            }
        } else {
            self.try_normal_insertion(
                transaction,
                source_node_index,
                target_node_index,
                node_type,
                factory,
                is_recurrent,
            )
        }
    }

    fn try_edge_insertion<C>(
        &self,
        transaction: &mut GraphTransaction<C>,
        source_node: usize,
        target_node: usize,
        node_type: &NodeType,
        factory: &CellStore<C>,
    ) -> bool
    where
        C: Clone + Default + PartialEq + NodeCell,
    {
        let new_source_edge_index = transaction.graph.nodes.len();
        let new_node_index = transaction.graph.nodes.len() + 1;
        let new_target_edge_index = transaction.graph.nodes.len() + 2;

        let source_node = transaction.graph.nodes[source_node].index;
        let target_node = transaction.graph.nodes[target_node].index;

        if transaction.graph[target_node].is_locked() {
            let edge = factory.new_instance((
                new_source_edge_index,
                transaction.graph.nodes[source_node].node_type,
            ));
            let new_node = factory.new_instance((new_node_index, *node_type));

            let edge_index = transaction.add_node(edge);
            let node_index = transaction.add_node(new_node);

            transaction.attach(source_node, node_index);
            transaction.attach(node_index, edge_index);
            transaction.attach(edge_index, target_node);
            transaction.detach(source_node, target_node);
        } else {
            let new_source_edge = factory.new_instance((
                new_source_edge_index,
                transaction.graph.nodes[source_node].node_type,
            ));
            let new_node = factory.new_instance((new_node_index, *node_type));
            let new_target_edge = factory.new_instance((
                new_target_edge_index,
                transaction.graph.nodes[target_node].node_type,
            ));

            transaction.add_node(new_source_edge);
            transaction.add_node(new_node);
            transaction.add_node(new_target_edge);

            transaction.attach(source_node, new_source_edge_index);
            transaction.attach(new_source_edge_index, new_node_index);
            transaction.attach(new_node_index, new_target_edge_index);
            transaction.attach(new_target_edge_index, target_node);
        }

        self.complete_node_arity(transaction, new_node_index, false)
    }

    fn try_backward_edge_insertion<C>(
        &self,
        transaction: &mut GraphTransaction<C>,
        source_idx: usize,
        target_idx: usize,
        node_type: &NodeType,
        factory: &CellStore<C>,
    ) -> bool
    where
        C: Clone + Default + PartialEq + NodeCell,
    {
        let new_source_edge_index = transaction.graph.nodes.len();
        let new_node_index = transaction.graph.nodes.len() + 1;
        let new_target_edge_index = transaction.graph.nodes.len() + 2;
        let recurrent_edge_index = transaction.graph.nodes.len() + 3;

        // Get node info before mutations
        let source_node_type = transaction.graph.nodes[source_idx].node_type;
        let source_is_recurrent = transaction.graph.nodes[source_idx].is_recurrent();
        let source_incoming = transaction.graph.nodes[source_idx].incoming.clone();
        let source_outgoing = transaction.graph.nodes[source_idx].outgoing.clone();
        let target_is_locked = transaction.graph.nodes[target_idx].is_locked();

        if source_node_type == NodeType::Edge && node_type != &NodeType::Edge {
            let incoming_idx = *source_incoming.iter().next().unwrap();
            let outgoing_idx = *source_outgoing.iter().next().unwrap();

            if target_is_locked {
                let mut new_node = factory.new_instance((new_node_index, *node_type));
                new_node.direction = Direction::Backward;
                let new_source_edge =
                    factory.new_instance((new_source_edge_index, source_node_type));
                let new_target_edge =
                    factory.new_instance((new_target_edge_index, source_node_type));

                transaction.add_node(new_source_edge);
                transaction.add_node(new_node);
                transaction.add_node(new_target_edge);

                transaction.attach(incoming_idx, new_node_index);
                transaction.attach(new_node_index, new_source_edge_index);
                transaction.attach(new_source_edge_index, new_node_index);
                transaction.attach(new_node_index, new_target_edge_index);
                transaction.attach(new_target_edge_index, outgoing_idx);
                transaction.detach(incoming_idx, outgoing_idx);
            } else if !source_is_recurrent {
                let mut new_node = factory.new_instance((new_node_index, *node_type));
                new_node.direction = Direction::Backward;
                let new_source_edge =
                    factory.new_instance((new_source_edge_index, source_node_type));
                let new_target_edge =
                    factory.new_instance((new_target_edge_index, source_node_type));
                let recurrent_edge = factory.new_instance((recurrent_edge_index, source_node_type));

                transaction.add_node(new_source_edge);
                transaction.add_node(new_node);
                transaction.add_node(new_target_edge);
                transaction.add_node(recurrent_edge);

                transaction.attach(incoming_idx, new_source_edge_index);
                transaction.attach(new_source_edge_index, new_node_index);
                transaction.attach(new_node_index, new_target_edge_index);
                transaction.attach(new_target_edge_index, outgoing_idx);
                transaction.attach(recurrent_edge_index, new_node_index);
                transaction.attach(new_node_index, recurrent_edge_index);
            } else {
                let mut new_node = factory.new_instance((new_node_index, *node_type));
                new_node.direction = Direction::Backward;
                let new_source_edge =
                    factory.new_instance((new_source_edge_index, source_node_type));
                let new_target_edge =
                    factory.new_instance((new_target_edge_index, source_node_type));

                transaction.add_node(new_source_edge);
                transaction.add_node(new_node);
                transaction.add_node(new_target_edge);

                transaction.attach(incoming_idx, new_source_edge_index);
                transaction.attach(new_source_edge_index, new_node_index);
                transaction.attach(new_node_index, new_target_edge_index);
                transaction.attach(new_target_edge_index, outgoing_idx);
            }

            self.complete_node_arity(transaction, new_node_index, true)
        } else {
            if !&transaction.graph.can_connect(source_idx, target_idx, true) {
                return false;
            }

            let mut new_node = factory.new_instance((new_node_index, *node_type));
            new_node.direction = Direction::Backward;
            transaction.add_node(new_node);

            transaction.attach(source_idx, new_node_index);
            transaction.attach(new_node_index, target_idx);
            transaction.detach(source_idx, target_idx);

            self.complete_node_arity(transaction, new_node_index, true)
        }
    }

    fn try_normal_insertion<C>(
        &self,
        transaction: &mut GraphTransaction<C>,
        source_node: usize,
        target_node: usize,
        node_type: &NodeType,
        factory: &CellStore<C>,
        is_recurrent: bool,
    ) -> bool
    where
        C: Clone + Default + PartialEq + NodeCell,
    {
        if !&transaction
            .graph
            .can_connect(source_node, target_node, is_recurrent)
        {
            return false;
        }

        let mut new_node = factory.new_instance((transaction.graph.nodes.len(), *node_type));

        if is_recurrent {
            new_node.direction = Direction::Backward;
        }

        let node_index = transaction.add_node(new_node);
        transaction.attach(source_node, node_index);
        transaction.attach(node_index, target_node);
        transaction.detach(source_node, target_node);

        self.complete_node_arity(transaction, node_index, is_recurrent)
    }

    fn complete_node_arity<C>(
        &self,
        transaction: &mut GraphTransaction<C>,
        node_index: usize,
        is_recurrent: bool,
    ) -> bool
    where
        C: Clone + Default + PartialEq + NodeCell,
    {
        let arity = transaction.graph.nodes[node_index].value.arity();

        match arity {
            Arity::Any | Arity::Zero => {
                return true;
            }
            Arity::Exact(arity) => {
                for _ in 0..arity - 1 {
                    let other_source_node = transaction.graph.random_source_node();
                    if transaction.graph.can_connect(
                        other_source_node.index,
                        node_index,
                        is_recurrent,
                    ) {
                        transaction.attach(other_source_node.index, node_index);
                    }
                }
            }
        }

        let effects = transaction.effects.clone();

        for idx in effects {
            let node_cycles = transaction.graph.get_cycles(idx);

            if node_cycles.is_empty() {
                transaction.change_direction(idx, Direction::Forward);
            } else {
                for cycle_idx in node_cycles {
                    transaction.change_direction(cycle_idx, Direction::Backward);
                }
            }
        }

        true
    }
}
