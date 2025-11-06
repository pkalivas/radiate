//! Transactional editing for `Graph<T>`.
//!
//! A `GraphTransaction` records reversible mutations (add/remove nodes and edges,
//! direction changes), can mark cycles, and validates the graph on commit. If a commit
//! fails, the graph is automatically rolled back and "replay" steps are returned so
//! you can re-apply the same changes later (e.g., after adjustments or in a new context).
//!
//! Key features:
//! - Atomic commit/rollback semantics
//! - Cycle marking: nodes in detected cycles are marked `Direction::Backward`
//! - Validation integration via `Valid`
//! - Deterministic tests via `random_provider::set_seed(...)`
//!
//! Typical flow:
//! 1) Build with `add_node`/`attach`/`detach`/`change_direction`
//! 2) `commit()` or `commit_with(...)`
//! 3) On invalid commit, use returned `replay` to re-apply later with `replay(...)`

use super::{Direction, Graph, GraphNode};
use crate::{Arity, NodeType, node::Node};
use radiate_core::{Valid, random_provider};
use std::{collections::BTreeSet, fmt::Debug, ops::Deref};

const SOURCE_NODE_TYPES: &[NodeType] = &[NodeType::Input, NodeType::Vertex, NodeType::Edge];
const TARGET_NODE_TYPES: &[NodeType] = &[NodeType::Output, NodeType::Vertex, NodeType::Edge];

/// A single reversible mutation applied during a transaction.
///
/// This is a structural log of intent (what you asked to do), used for introspection and
/// reporting back to callers. Reversal is handled by `rollback()` which produces [ReplayStep]s.
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

/// A replayable step produced by `rollback()` to restore the effects that were undone.
///
/// Unlike [MutationStep], this is designed for re-applying operational effects on another
/// transaction via `replay(...)`. For `AddNode`, the index is informational; re-application
/// uses the provided [GraphNode] if present.
#[derive(Clone)]
pub enum ReplayStep<T> {
    AddNode(usize, Option<GraphNode<T>>),
    AddEdge(usize, usize),
    RemoveEdge(usize, usize),
    DirectionChange(usize, Direction),
}

/// Result of finalizing a transaction.
///
/// - `Valid(steps)`: the graph remained valid after cycle marking (and optional custom validation),
///   and no rollback occurred.
/// - `Invalid(steps, replay)`: validation failed; the graph was rolled back to its original state,
///   and `replay` contains steps to re-apply the effects elsewhere or later.
pub enum TransactionResult<T> {
    Valid(Vec<MutationStep>),
    Invalid(Vec<MutationStep>, Vec<ReplayStep<T>>),
}

/// A declarative plan for inserting a node between two nodes.
///
/// Consumers must execute these steps themselves (e.g., with `attach`/`detach`) before committing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InsertStep {
    Detach(usize, usize),
    Connect(usize, usize),
    Invalid,
}

/// Tracks reversible changes to a `Graph<T>` and provides commit/rollback.
///
/// Usage:
/// - Mutate via `add_node`, `attach`, `detach`, `change_direction`.
/// - Call `commit()` or `commit_with(...)`.
/// - On invalid commit, the graph is rolled back and you receive `replay` steps you can pass to
///   `replay(...)` in a fresh transaction.
pub struct GraphTransaction<'a, T> {
    graph: &'a mut Graph<T>,
    steps: Vec<MutationStep>,
    effects: BTreeSet<usize>,
}

impl<'a, T> GraphTransaction<'a, T> {
    pub fn new(graph: &'a mut Graph<T>) -> Self {
        GraphTransaction {
            graph,
            steps: Vec::with_capacity(5),
            effects: BTreeSet::new(),
        }
    }

    /// Finalize the transaction:
    /// - Marks cycles (`set_cycles()`).
    /// - Validates with `Graph<T>: Valid`.
    /// - If valid, returns `Valid(steps)`.
    /// - If invalid, rolls back and returns `Invalid(steps, replay)`.
    pub fn commit(self) -> TransactionResult<T> {
        self.commit_internal::<fn(&Graph<T>) -> bool>(None)
    }

    /// Like `commit()`, but also requires `validator(&Graph<T>)` to pass.
    ///
    /// This is useful for domain-specific acceptance checks in addition to structural validity.
    pub fn commit_with(self, validator: impl Fn(&Graph<T>) -> bool) -> TransactionResult<T> {
        self.commit_internal(Some(validator))
    }

    /// Append a node to the graph and record the change. Returns the new node's index.
    pub fn add_node(&mut self, node: impl Into<GraphNode<T>>) -> usize {
        let index = self.graph.len();
        self.steps.push(MutationStep::AddNode(index));
        self.graph.push(node);
        self.effects.insert(index);
        index
    }

    /// Create an edge from `from` to `to` and record the change.
    pub fn attach(&mut self, from: usize, to: usize) {
        self.steps.push(MutationStep::AddEdge(from, to));
        self.graph.attach(from, to);
        self.effects.insert(from);
        self.effects.insert(to);
    }

    /// Remove an edge from `from` to `to` and record the change.
    pub fn detach(&mut self, from: usize, to: usize) {
        self.steps.push(MutationStep::RemoveEdge(from, to));
        self.graph.detach(from, to);
        self.effects.insert(from);
        self.effects.insert(to);
    }

    /// Change the direction of the node at `index` if it differs from its current direction.
    ///
    /// Records the previous direction so the change can be rolled back or replayed.
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

    /// Undo all recorded changes in reverse order, mutating the graph back to its original state.
    ///
    /// Returns a sequence of `ReplayStep`s that can be fed to `replay(...)` on a new transaction
    /// to re-apply the same operational effects.
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

    /// Apply `ReplayStep`s (typically from a prior `rollback()`) to this transaction/graph.
    ///
    /// Steps are recorded as normal mutations (so they can be committed or rolled back again).
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

    /// Mark cycle participation for nodes touched in this transaction:
    /// - Nodes without cycles are set to `Direction::Forward`.
    /// - Nodes in cycles (via `get_cycles(idx)`) are set to `Direction::Backward`.
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

    /// Compute the steps needed to insert `new_node_idx` between `source_idx` and `target_idx`.
    ///
    /// Behavior:
    /// - If `new_node` has `Arity::Zero` and `target` is not locked, connect `new_node -> target`.
    /// - If `source` is an `Edge`, re-route its single outgoing through `new_node`.
    /// - If `target` is an `Edge` or is locked, detach one incoming and rewire via `new_node`.
    /// - Otherwise connect `source -> new_node -> target`.
    #[inline]
    pub fn get_insertion_steps(
        &self,
        source_idx: usize,
        target_idx: usize,
        new_node_idx: usize,
    ) -> Vec<InsertStep> {
        let target_node = self.graph.get(target_idx).unwrap();
        let source_node = self.graph.get(source_idx).unwrap();
        let new_node = self.graph.get(new_node_idx).unwrap();

        let mut steps = Vec::with_capacity(4);

        let source_is_edge = source_node.node_type() == NodeType::Edge;
        let target_is_edge = target_node.node_type() == NodeType::Edge;
        let new_node_arity = new_node.arity();

        if new_node_arity == Arity::Zero && !target_node.is_locked() {
            steps.push(InsertStep::Connect(new_node_idx, target_idx));
            return steps;
        }

        if source_is_edge {
            let source_outgoing = *random_provider::choose(&source_node.outgoing());

            if source_outgoing == new_node_idx {
                steps.push(InsertStep::Connect(source_idx, new_node_idx));
            } else {
                steps.push(InsertStep::Connect(source_idx, new_node_idx));
                steps.push(InsertStep::Connect(new_node_idx, source_outgoing));
                steps.push(InsertStep::Detach(source_idx, source_outgoing));
            }
        } else if target_is_edge || target_node.is_locked() {
            let target_incoming = *random_provider::choose(&target_node.incoming());

            if target_incoming == new_node_idx {
                steps.push(InsertStep::Connect(target_incoming, new_node_idx));
            } else {
                steps.push(InsertStep::Connect(target_incoming, new_node_idx));
                steps.push(InsertStep::Connect(new_node_idx, target_idx));
                steps.push(InsertStep::Detach(target_incoming, target_idx));
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
        self.random_node_of_type(SOURCE_NODE_TYPES)
    }
    /// Get a random node that can be used as a target node for a connection.
    /// A target node can be either an output or a vertex node.
    #[inline]
    pub fn random_target_node(&self) -> Option<&GraphNode<T>> {
        self.random_node_of_type(TARGET_NODE_TYPES)
    }
    /// Helper functions to get a random node of the specified type. If no nodes of the specified
    /// type are found, the function will try to get a random node of a different type.
    /// If no nodes are found, the function will panic.
    #[inline]
    fn random_node_of_type(&self, node_types: &[NodeType]) -> Option<&GraphNode<T>> {
        if node_types.is_empty() {
            return None;
        }

        let gene_node_type = random_provider::choose(&node_types);

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
                    .collect::<Vec<NodeType>>()
                    .as_slice(),
            );
        }

        Some(*random_provider::choose(&genes))
    }

    fn commit_internal<F: Fn(&Graph<T>) -> bool>(
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
}

impl<T> Deref for GraphTransaction<'_, T> {
    type Target = Graph<T>;

    fn deref(&self) -> &Self::Target {
        self.graph
    }
}

#[cfg(test)]
mod tests {
    use super::{GraphTransaction, InsertStep, MutationStep, TransactionResult};
    use crate::collections::graphs::{Direction, Graph, GraphNode};
    use crate::{Arity, Node, NodeType};
    use radiate_core::{Valid, random_provider};

    fn assert_has_direction_change(steps: &[MutationStep], idxs: &[usize]) {
        let mut seen = vec![];
        for s in steps {
            if let MutationStep::DirectionChange { index, .. } = s {
                seen.push(*index);
            }
        }
        for idx in idxs {
            assert!(
                seen.contains(idx),
                "Expected DirectionChange for node {} not found in steps: {:?}",
                idx,
                steps
            );
        }
    }

    #[test]
    fn commit_valid_add_and_attach() {
        let mut g = Graph::<i32>::default();
        let mut tx = GraphTransaction::new(&mut g);

        let i = tx.add_node((0, NodeType::Input, 0));
        let o = tx.add_node((1, NodeType::Output, 1));
        tx.attach(i, o);

        match tx.commit() {
            TransactionResult::Valid(steps) => {
                assert_eq!(steps.len(), 3);
                assert!(matches!(steps[0], MutationStep::AddNode(0)));
                assert!(matches!(steps[1], MutationStep::AddNode(1)));
                assert!(matches!(steps[2], MutationStep::AddEdge(0, 1)));
                assert!(g.is_valid());
                assert_eq!(g[0].outgoing().len(), 1);
                assert_eq!(g[1].incoming().len(), 1);
                assert_eq!(g[0].direction(), Direction::Forward);
                assert_eq!(g[1].direction(), Direction::Forward);
            }
            _ => panic!("expected Valid"),
        }
    }

    #[test]
    fn commit_invalid_rolls_back_and_replay_restores() {
        let mut g = Graph::<i32>::default();

        // Build: Input -> Vertex(arity=2) -> Output (invalid: vertex missing one incoming)
        let mut tx = GraphTransaction::new(&mut g);
        let input = tx.add_node((0, NodeType::Input, 0));
        let vertex = tx.add_node((1, NodeType::Vertex, 1, Arity::Exact(2)));
        let output = tx.add_node((2, NodeType::Output, 2));

        tx.attach(input, vertex);
        tx.attach(vertex, output);

        let (steps, replay) = match tx.commit() {
            TransactionResult::Invalid(steps, replay) => (steps, replay),
            _ => panic!("expected Invalid"),
        };

        // Graph must be rolled back to original state (empty)
        assert_eq!(g.len(), 0, "graph should be rolled back to empty");
        assert!(g.is_valid());

        // Reapply the changes using replay steps
        let mut tx2 = GraphTransaction::new(&mut g);
        tx2.replay(replay);

        assert_eq!(g.len(), 3);
        assert_eq!(g[0].node_type(), NodeType::Input);
        assert_eq!(g[1].node_type(), NodeType::Vertex);
        assert_eq!(g[2].node_type(), NodeType::Output);
        assert!(g[0].outgoing().contains(&1));
        assert!(g[1].incoming().contains(&0));
        assert!(g[1].outgoing().contains(&2));
        assert!(g[2].incoming().contains(&1));

        // Sanity: original mutation steps captured structure we tried
        assert!(steps.iter().any(|s| matches!(s, MutationStep::AddNode(0))));
        assert!(steps.iter().any(|s| matches!(s, MutationStep::AddNode(1))));
        assert!(steps.iter().any(|s| matches!(s, MutationStep::AddNode(2))));
        assert!(
            steps
                .iter()
                .any(|s| matches!(s, MutationStep::AddEdge(0, 1)))
        );
        assert!(
            steps
                .iter()
                .any(|s| matches!(s, MutationStep::AddEdge(1, 2)))
        );
    }

    #[test]
    fn commit_sets_cycles_and_marks_backward() {
        let mut g = Graph::<i32>::default();
        let mut tx = GraphTransaction::new(&mut g);

        let a = tx.add_node((0, NodeType::Vertex, 10));
        let b = tx.add_node((1, NodeType::Vertex, 20));
        tx.attach(a, b);
        tx.attach(b, a); // creates cycle {0,1}

        match tx.commit() {
            TransactionResult::Valid(steps) => {
                assert!(g.is_valid());
                // Both nodes in the cycle should be marked Backward
                assert_eq!(g[0].direction(), Direction::Backward);
                assert_eq!(g[1].direction(), Direction::Backward);
                // And the mutation steps should include direction changes
                assert_has_direction_change(&steps, &[0, 1]);
            }
            _ => panic!("expected Valid"),
        }
    }

    #[test]
    fn insertion_steps_new_zero_arity_connects_to_target_when_unlocked() {
        let mut g = Graph::<i32>::default();
        let mut tx = GraphTransaction::new(&mut g);

        let src = tx.add_node((0, NodeType::Input, 0));
        let tgt = tx.add_node((1, NodeType::Vertex, 1)); // Arity::Any => not locked
        let newn = tx.add_node((2, NodeType::Input, 2)); // Arity::Zero

        let steps = tx.get_insertion_steps(src, tgt, newn);
        assert_eq!(steps, vec![InsertStep::Connect(newn, tgt)]);
    }

    #[test]
    fn insertion_steps_source_is_edge_with_single_outgoing_equal_new() {
        let mut g = Graph::<i32>::default();
        let mut tx = GraphTransaction::new(&mut g);

        // source edge with outgoing already pointing to new node
        let source = tx.add_node(
            GraphNode::with_arity(0, NodeType::Edge, 0, Arity::Exact(1)).with_outgoing([2]),
        );
        let target = tx.add_node((1, NodeType::Vertex, 1));
        let newn = tx.add_node((2, NodeType::Vertex, 2));

        let steps = tx.get_insertion_steps(source, target, newn);
        assert_eq!(steps, vec![InsertStep::Connect(source, newn)]);
    }

    #[test]
    fn insertion_steps_source_is_edge_redirects_through_new() {
        let mut g = Graph::<i32>::default();
        let mut tx = GraphTransaction::new(&mut g);

        // source edge with single outgoing to target (not new)
        let source = tx.add_node(
            GraphNode::with_arity(0, NodeType::Edge, 0, Arity::Exact(1)).with_outgoing([1]),
        );
        let target = tx.add_node((1, NodeType::Vertex, 1));
        let newn = tx.add_node((2, NodeType::Vertex, 2));

        let steps = tx.get_insertion_steps(source, target, newn);
        assert_eq!(
            steps,
            vec![
                InsertStep::Connect(source, newn),
                InsertStep::Connect(newn, target),
                InsertStep::Detach(source, target),
            ]
        );
    }

    #[test]
    fn insertion_steps_target_locked_prefers_detach_rewire() {
        let mut g = Graph::<i32>::default();
        let mut tx = GraphTransaction::new(&mut g);

        // target is "locked": Arity::Exact(1) with exactly one incoming; ensure not an Edge type
        // by keeping outgoing empty.
        let source = tx.add_node((0, NodeType::Vertex, 0));
        let target = tx.add_node(
            GraphNode::with_arity(1, NodeType::Vertex, 1, Arity::Exact(1)).with_incoming([0]),
        );
        let newn = tx.add_node((2, NodeType::Vertex, 2));

        let steps = tx.get_insertion_steps(source, target, newn);
        assert_eq!(
            steps,
            vec![
                InsertStep::Connect(source, newn),
                InsertStep::Connect(newn, target),
                InsertStep::Detach(source, target),
            ]
        );
    }

    #[test]
    fn random_node_helpers_can_return_edges_when_only_edges_exist() {
        random_provider::set_seed(1337);

        let mut g = Graph::<i32>::default();
        let mut tx = GraphTransaction::new(&mut g);

        // Only edge nodes exist; helpers should still return something (and it will be an Edge).
        tx.add_node((0, NodeType::Edge, 0, Arity::Exact(1)));
        tx.add_node((1, NodeType::Edge, 1, Arity::Exact(1)));

        let src = tx.random_source_node().unwrap();
        let tgt = tx.random_target_node().unwrap();

        assert_eq!(src.node_type(), NodeType::Edge);
        assert_eq!(tgt.node_type(), NodeType::Edge);
    }
}
