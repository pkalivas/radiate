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
//! - Repair of invalid nodes (e.g., missing connections) before final validation in `try_commit()`
//!
//! Typical flow:
//! 1) Build with `push(...)`, `attach(...)`, `detach(...)`, `change_direction(...)`
//! 2) `commit()`, `commit_with(...)`, or `try_commit()` to finalize
//! 3) On invalid commit, use returned `replay` to re-apply later with `replay(...)`
use super::{Direction, Graph, GraphNode, NodeIdx, cycles::Sccs};
use crate::{Arity, NodeType, graphs::node::InnovationId, node::Node};
use radiate_core::{RdRand, Valid, random_provider};
use radiate_utils::SortedBuffer;
use std::{fmt::Debug, ops::Deref};

const SOURCE_NODE_TYPES: &[NodeType] = &[NodeType::Input, NodeType::Vertex];
const TARGET_NODE_TYPES: &[NodeType] = &[NodeType::Output, NodeType::Vertex, NodeType::Edge];
const MAX_REPAIR_ATTEMPTS: usize = 10;
const MAX_SOURCE_ATTEMPTS: usize = 10;
const MAX_REJECTION_DRAWS: usize = 8;

/// A single reversible mutation applied during a transaction.
///
/// This is a structural log of intent (what you asked to do), used for introspection and
/// reporting back to callers. Reversal is handled by `rollback()` which produces [ReplayStep]s.
#[derive(Debug, Clone)]
pub enum MutationStep {
    AddNode(NodeIdx),
    AddEdge(NodeIdx, NodeIdx),
    RemoveEdge(NodeIdx, NodeIdx),
    DirectionChange {
        index: NodeIdx,
        previous_direction: Direction,
    },
    InnovationChange {
        node_idx: NodeIdx,
        previous_innovation: Option<InnovationId>,
    },
}

/// A replayable step produced by `rollback()` to restore the effects that were undone.
///
/// Unlike [MutationStep], this is designed for re-applying operational effects on another
/// transaction via `replay(...)`. For `AddNode`, the index is informational; re-application
/// uses the provided [GraphNode] if present.
#[derive(Clone)]
pub enum ReplayStep<T> {
    AddNode(NodeIdx, Option<GraphNode<T>>),
    AddEdge(NodeIdx, NodeIdx),
    RemoveEdge(NodeIdx, NodeIdx),
    DirectionChange(NodeIdx, Direction),
    InnovationChange(NodeIdx, Option<InnovationId>),
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

impl<T> TransactionResult<T> {
    pub fn is_valid(&self) -> bool {
        matches!(self, TransactionResult::Valid(_))
    }

    pub fn is_invalid(&self) -> bool {
        matches!(self, TransactionResult::Invalid(_, _))
    }

    pub fn replay(&self, graph: &mut Graph<T>)
    where
        T: Clone,
    {
        if let TransactionResult::Invalid(_, replay_steps) = self {
            let mut transaction = GraphTransaction::new(graph);
            transaction.replay(replay_steps.clone());
        }
    }
}

/// Tracks reversible changes to a `Graph<T>` and provides commit/rollback.
///
/// Usage:
/// - Mutate via `push(...)`, `attach(...)`, `detach(...)`, `change_direction(...)`.
/// - Call `commit()`, `commit_with(...)`, or `try_commit()` to finalize.
/// - On invalid commit, the graph is rolled back and you receive `replay` steps you can pass to
///   `replay(...)` in a fresh transaction.
pub struct GraphTransaction<'a, T> {
    graph: &'a mut Graph<T>,
    sccs: Option<&'a mut Sccs>,
    steps: Vec<MutationStep>,
}

impl<'a, T> GraphTransaction<'a, T> {
    pub fn new(graph: &'a mut Graph<T>) -> Self {
        GraphTransaction {
            graph,
            sccs: None,
            steps: Vec::with_capacity(5),
        }
    }

    pub fn with_sccs(&mut self, sccs: &'a mut Sccs) {
        self.sccs = Some(sccs);
    }

    /// Finalize the transaction:
    /// - Marks cycles (`set_cycles()`).
    /// - Validates with `Graph<T>: Valid`.
    /// - If valid, returns `Valid(steps)`.
    /// - If invalid, rolls back and returns `Invalid(steps, replay)`.
    pub fn commit(self) -> TransactionResult<T> {
        self.commit_internal::<fn(&Graph<T>) -> bool>(None, true)
    }

    /// Like `commit()`, but also requires `validator(&Graph<T>)` to pass.
    ///
    /// This is useful for domain-specific acceptance checks in addition to structural validity.
    pub fn commit_with(self, validator: impl Fn(&Graph<T>) -> bool) -> TransactionResult<T> {
        self.commit_internal(Some(validator), true)
    }

    /// Attempt to commit the transaction, repairing invalid nodes if possible.
    /// - Calls `repair_invalid_nodes()` up to `MAX_REPAIR_ATTEMPTS` times.
    ///
    /// Note: This may produce different graphs on each call due to possible random repairs.
    pub fn try_commit(mut self) -> TransactionResult<T> {
        let mut repaired = false;
        let mut attempts = 0;

        self.set_cycles();
        while !repaired && attempts < MAX_REPAIR_ATTEMPTS {
            repaired = self.repair_invalid_nodes();
            if repaired {
                self.set_cycles();
            }

            attempts += 1;
        }

        self.commit_internal::<fn(&Graph<T>) -> bool>(None, false)
    }

    /// Append a node to the graph and record the change. Returns the new node's index.
    pub fn push(&mut self, node: impl Into<GraphNode<T>>) -> NodeIdx {
        let index = NodeIdx::from_usize(self.graph.len());
        self.steps.push(MutationStep::AddNode(index));
        self.graph.push(node);
        index
    }

    /// Create an edge from `from` to `to` and record the change.
    pub fn attach(&mut self, from: impl Into<NodeIdx>, to: impl Into<NodeIdx>) {
        let (from, to) = (from.into(), to.into());
        self.steps.push(MutationStep::AddEdge(from, to));
        self.graph.attach(from, to);
    }

    /// Remove an edge from `from` to `to` and record the change.
    ///
    /// Does nothing (and records nothing) if the edge doesn't exist, so a rollback never
    /// attaches an edge that wasn't there to begin with.
    pub fn detach(&mut self, from: impl Into<NodeIdx>, to: impl Into<NodeIdx>) {
        let (from, to) = (from.into(), to.into());
        if self.graph[from].outgoing().binary_search(&to).is_err() {
            return;
        }

        self.steps.push(MutationStep::RemoveEdge(from, to));
        self.graph.detach(from, to);
    }

    /// Change the direction of the node at `index` if it differs from its current direction.
    ///
    /// Records the previous direction so the change can be rolled back or replayed.
    pub fn change_direction(&mut self, index: impl Into<NodeIdx>, direction: Direction) {
        let index = index.into();
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

    pub fn set_innovation(
        &mut self,
        node_idx: impl Into<NodeIdx>,
        innovation: Option<InnovationId>,
    ) {
        let node_idx = node_idx.into();
        if let Some(node) = self.graph.get_mut(node_idx) {
            let previous_innovation = node.innovation();
            node.set_innovation(innovation);
            self.steps.push(MutationStep::InnovationChange {
                node_idx,
                previous_innovation,
            });
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
                    let index = NodeIdx::from_usize(self.graph.len());
                    replay_steps.push(ReplayStep::AddNode(index, added_node));
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
                MutationStep::InnovationChange {
                    node_idx,
                    previous_innovation,
                } => {
                    if let Some(node) = self.graph.get_mut(node_idx) {
                        let current_innovation = node.innovation();
                        node.set_innovation(previous_innovation);
                        replay_steps
                            .push(ReplayStep::InnovationChange(node_idx, current_innovation));
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
                        self.push(node);
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
                ReplayStep::InnovationChange(node_idx, innovation) => {
                    self.set_innovation(node_idx, innovation);
                }
            }
        }
    }

    fn commit_internal<F: Fn(&Graph<T>) -> bool>(
        mut self,
        validator: Option<F>,
        set_cycles: bool,
    ) -> TransactionResult<T> {
        if set_cycles {
            self.set_cycles();
        }

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

    fn set_cycles(&mut self) {
        match self.sccs.take() {
            Some(sccs) => {
                self.mark_cycles(sccs);
                self.sccs = Some(sccs);
            }
            None => self.mark_cycles(&mut Sccs::default()),
        }
    }

    fn mark_cycles(&mut self, sccs: &mut Sccs) {
        sccs.find_in(self.graph.as_ref());

        for (index, direction) in sccs.directions() {
            self.change_direction(index, direction);
        }
    }
}

/// Methods for manipulating the graph structure within a transaction.
impl<'a, T> GraphTransaction<'a, T> {
    pub fn push_and_fill(
        &mut self,
        new_node: GraphNode<T>,
        allow_recurrent: bool,
    ) -> Option<NodeIdx> {
        random_provider::with_rng(|rand| {
            self.push_and_fill_with_random(new_node, allow_recurrent, rand)
        })
    }

    pub fn push_and_fill_with_random(
        &mut self,
        new_node: GraphNode<T>,
        allow_recurrent: bool,
        rand: &mut RdRand,
    ) -> Option<NodeIdx> {
        let target_idx = self.random_target_node(rand)?;
        let arity = new_node.arity();
        let node_idx = self.push(new_node);

        let mut needed = match arity {
            Arity::Exact(n) => n,
            _ => 1,
        };

        if self.graph.len() < needed {
            return None;
        }

        let target_locked = self.is_locked(target_idx);

        if target_locked {
            let random_incoming = self.random_incoming_from(target_idx, rand)?;

            self.attach(random_incoming, node_idx);
            self.attach(node_idx, target_idx);
            self.detach(random_incoming, target_idx);

            needed -= 1;
        } else {
            self.attach(node_idx, target_idx);

            if arity == Arity::Zero {
                needed -= 1;
            }
        }

        if needed == 0 {
            return Some(node_idx);
        }

        let mut attempts = 0;
        let mut sorted = SortedBuffer::new();
        while sorted.len() < needed && attempts < MAX_SOURCE_ATTEMPTS {
            if let Some(idx) = self.random_source_node(rand) {
                if !allow_recurrent && (idx == target_idx || idx == node_idx) {
                    continue;
                }

                sorted.insert(idx);
            }

            attempts += 1;
        }

        for &src in sorted.iter() {
            if self.graph.are_attached(src, node_idx) {
                continue;
            }

            self.attach(src, node_idx);

            if self.graph.are_attached(src, target_idx) && !target_locked && rand.bool(0.5) {
                self.detach(src, target_idx);
            }

            needed -= 1;
        }

        if needed == 0 {
            return Some(node_idx);
        }

        let random_incoming = self.random_incoming_from(target_idx, rand)?;

        if random_incoming == node_idx && allow_recurrent {
            self.attach(node_idx, node_idx);
            needed -= 1;
        } else if self.node_type(random_incoming)? == NodeType::Edge {
            self.detach(random_incoming, target_idx);
            self.attach(random_incoming, node_idx);
            needed -= 1;
        } else {
            self.attach(random_incoming, node_idx);
            needed -= 1;
        }

        if needed > 0 {
            return None;
        }

        Some(node_idx)
    }
}

/// Random access functionality for selecting nodes from the graph.
impl<'a, T> GraphTransaction<'a, T> {
    fn random_incoming_from(&self, node_idx: NodeIdx, rand: &mut RdRand) -> Option<NodeIdx> {
        let incoming_indicies = self.graph.get(node_idx)?.incoming();
        if incoming_indicies.is_empty() {
            None
        } else {
            Some(*rand.choose(incoming_indicies))
        }
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
    fn random_source_node(&self, rand: &mut RdRand) -> Option<NodeIdx> {
        self.random_node_of_type(SOURCE_NODE_TYPES, rand)
            .map(|node| node.index())
    }

    /// Get a random node that can be used as a target node for a connection.
    /// A target node can be either an output or a vertex node.
    #[inline]
    fn random_target_node(&self, rand: &mut RdRand) -> Option<NodeIdx> {
        self.random_node_of_type(TARGET_NODE_TYPES, rand)
            .map(|node| node.index())
    }

    #[inline]
    fn random_node_of_type(
        &self,
        node_types: &[NodeType],
        rand: &mut RdRand,
    ) -> Option<&GraphNode<T>> {
        let nodes = self.graph.as_ref();
        if nodes.is_empty() || node_types.is_empty() {
            return None;
        }

        // Eligible nodes are nearly all of the graph, so a few draws almost always land.
        for _ in 0..MAX_REJECTION_DRAWS {
            let node = &nodes[rand.range(0..nodes.len())];
            if node_types.contains(&node.node_type()) {
                return Some(node);
            }
        }

        // Rare (tiny or input-heavy graphs): count, then take the k-th eligible node.
        self.random_node_where(rand, |n| node_types.contains(&n.node_type()))
    }

    /// Get a random target node that satisfies the provided filter function.
    /// This is essentially a filtered version of the above function `random_target_node`.
    #[inline]
    fn random_target_node_where<F>(&self, rand: &mut RdRand, filter: F) -> Option<&GraphNode<T>>
    where
        F: Fn(&GraphNode<T>) -> bool,
    {
        self.random_node_where(rand, |node: &GraphNode<T>| {
            TARGET_NODE_TYPES.contains(&node.node_type()) && filter(node)
        })
    }

    /// Get a random source node that satisfies the provided filter function.
    /// This is essentially a filtered version of the above function `random_source_node`.
    #[inline]
    fn random_source_node_where<F>(&self, rand: &mut RdRand, filter: F) -> Option<&GraphNode<T>>
    where
        F: Fn(&GraphNode<T>) -> bool,
    {
        self.random_node_where(rand, |node: &GraphNode<T>| {
            SOURCE_NODE_TYPES.contains(&node.node_type()) && filter(node)
        })
    }

    /// Uniform over nodes matching `filter`, in two passes and with no allocation.
    #[inline]
    fn random_node_where<F>(&self, rand: &mut RdRand, filter: F) -> Option<&GraphNode<T>>
    where
        F: Fn(&GraphNode<T>) -> bool,
    {
        let count = self.iter().filter(|n| filter(n)).count();
        if count == 0 {
            return None;
        }

        let k = rand.range(0..count);
        self.iter().filter(|n| filter(n)).nth(k)
    }
}

/// Methods for repairing invalid nodes within the graph transaction.
impl<'a, T> GraphTransaction<'a, T> {
    fn repair_invalid_nodes(&mut self) -> bool {
        if self.is_valid() {
            return false;
        }

        let mut repaired = false;

        let invalid_nodes = self
            .iter()
            .filter(|node| !node.is_valid())
            .map(|n| n.index())
            .collect::<Vec<NodeIdx>>();

        for idx in invalid_nodes.iter() {
            let arity = self.graph[*idx].arity();
            match arity {
                Arity::Zero if self.repair_zero_arity_node(*idx) => {
                    repaired = true;
                }
                Arity::Exact(_) if self.repair_exact_arity_node(*idx) => {
                    repaired = true;
                }
                _ => {}
            }
        }

        repaired
    }

    fn repair_zero_arity_node(&mut self, node_idx: NodeIdx) -> bool {
        let node = self.graph.get(node_idx).unwrap();
        if node.arity() != Arity::Zero {
            return false;
        }

        if node.outgoing().is_empty() {
            let random_target = random_provider::with_rng(|rand| {
                self.random_target_node_where(rand, |n| !n.is_locked() && n.index() != node_idx)
                    .map(|n| n.index())
            });

            if let Some(target) = random_target {
                self.attach(node.index(), target);

                if !self.graph[node_idx].outgoing().is_empty() {
                    return true;
                }
            }
        }

        false
    }

    fn repair_exact_arity_node(&mut self, node_idx: NodeIdx) -> bool {
        let arity = self.graph[node_idx].arity();
        if let Arity::Exact(n) = arity {
            let current_incoming = self.graph[node_idx].incoming().len();
            if current_incoming < n {
                let needed = n - current_incoming;

                let available_sources = random_provider::with_rng(|rand| {
                    (0..needed)
                        .filter_map(|_| {
                            self.random_source_node_where(rand, |n| {
                                !n.is_locked() && n.index() != node_idx
                            })
                            .map(|n| n.index())
                        })
                        .collect::<Vec<NodeIdx>>()
                });

                for src in available_sources {
                    self.attach(src, node_idx);
                }

                if self.graph[node_idx].incoming().len() == n {
                    return true;
                }
            } else if current_incoming > n {
                let to_detach = current_incoming - n;

                let valid_incoming = self.graph[node_idx]
                    .incoming()
                    .iter()
                    .cloned()
                    .filter(|incoming| self.graph[*incoming].outgoing().len() > 1)
                    .collect::<Vec<NodeIdx>>();

                let rand_indices = random_provider::shuffled_indices(0..valid_incoming.len());
                let rand_indices = &rand_indices[0..to_detach];

                for &i in rand_indices.iter() {
                    let source_idx = valid_incoming[i];
                    self.detach(source_idx, node_idx);
                }

                if self.graph[node_idx].incoming().len() == n {
                    return true;
                }
            }
        }

        false
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
    use super::{GraphTransaction, MutationStep, TransactionResult};
    use crate::collections::NodeIdx;
    use crate::collections::graphs::{Direction, Graph, InnovationId};
    use crate::{Arity, Node, NodeType};
    use radiate_core::Valid;

    fn assert_has_direction_change(steps: &[MutationStep], idxs: &[u32]) {
        let mut seen = vec![];
        for s in steps {
            if let MutationStep::DirectionChange { index, .. } = s {
                seen.push(*index);
            }
        }
        for idx in idxs {
            assert!(
                seen.contains(&NodeIdx::new(*idx)),
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

        let i = tx.push((0, NodeType::Input, 0));
        let o = tx.push((1, NodeType::Output, 1));
        tx.attach(i, o);

        match tx.commit() {
            TransactionResult::Valid(steps) => {
                assert_eq!(steps.len(), 3);
                assert!(matches!(steps[0], MutationStep::AddNode(a) if a == 0));
                assert!(matches!(steps[1], MutationStep::AddNode(a) if a == 1));
                assert!(matches!(steps[2], MutationStep::AddEdge(a, b) if a == 0 && b == 1));
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
        let input = tx.push((0, NodeType::Input, 0));
        let vertex = tx.push((1, NodeType::Vertex, 1, Arity::Exact(2)));
        let output = tx.push((2, NodeType::Output, 2));

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
        assert!(g[0].outgoing().contains(&NodeIdx::new(1)));
        assert!(g[1].incoming().contains(&NodeIdx::new(0)));
        assert!(g[1].outgoing().contains(&NodeIdx::new(2)));
        assert!(g[2].incoming().contains(&NodeIdx::new(1)));

        // Sanity: original mutation steps captured structure we tried
        assert!(
            steps
                .iter()
                .any(|s| matches!(s, MutationStep::AddNode(a) if *a == 0))
        );
        assert!(
            steps
                .iter()
                .any(|s| matches!(s, MutationStep::AddNode(a) if *a == 1))
        );
        assert!(
            steps
                .iter()
                .any(|s| matches!(s, MutationStep::AddNode(a) if *a == 2))
        );
        assert!(
            steps
                .iter()
                .any(|s| matches!(s, MutationStep::AddEdge(a, b) if *a == 0 && *b == 1))
        );
        assert!(
            steps
                .iter()
                .any(|s| matches!(s, MutationStep::AddEdge(a, b) if *a == 1 && *b == 2))
        );
    }

    #[test]
    fn commit_sets_cycles_and_marks_backward() {
        let mut g = Graph::<i32>::default();
        let mut tx = GraphTransaction::new(&mut g);

        let a = tx.push((0, NodeType::Vertex, 10));
        let b = tx.push((1, NodeType::Vertex, 20));
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
    fn rollback_restores_previous_innovation_and_replay_reapplies() {
        let mut g = Graph::<i32>::default();
        let initial = InnovationId::new();
        let updated = InnovationId::new();

        {
            let mut tx = GraphTransaction::new(&mut g);
            let input = tx.push((0, NodeType::Input, 0));
            let output = tx.push((1, NodeType::Output, 1));
            tx.attach(input, output);
            tx.set_innovation(input, Some(initial));
            assert!(matches!(tx.commit(), TransactionResult::Valid(_)));
        }
        assert_eq!(g[0].innovation(), Some(initial));

        let replay = {
            let mut tx = GraphTransaction::new(&mut g);
            tx.set_innovation(0, Some(updated));
            match tx.commit_with(|_| false) {
                TransactionResult::Invalid(_, replay) => replay,
                _ => panic!("expected forced rejection"),
            }
        };

        assert_eq!(
            g[0].innovation(),
            Some(initial),
            "rollback should restore previous innovation, not clear it"
        );

        {
            let mut tx = GraphTransaction::new(&mut g);
            tx.replay(replay);
            assert!(matches!(tx.commit(), TransactionResult::Valid(_)));
        }
        assert_eq!(
            g[0].innovation(),
            Some(updated),
            "replay should reapply the innovation change"
        );
    }
}
