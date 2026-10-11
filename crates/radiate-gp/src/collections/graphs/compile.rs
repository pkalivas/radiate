//! Compiling a graph into a [GraphPlan]: which slot each node lives in, what each slot reads,
//! and the order to run them in. Structure only; [GraphKernel](super::GraphKernel) runs it.
//!
//! # Slots
//!
//! Every node gets a *slot*: its position in the plan, and its row in the kernel's
//! `slots × samples` value matrix.
//!
//! # What an edge reads
//!
//! The row evaluator's rule for what a node reads is: nodes run in the order given by
//! [GraphIterator::iter_topological], and each node reads whatever its source currently holds.
//! That gives every edge one of three meanings ([When]):
//!
//! ```text
//!   source runs earlier in the order   → Current:  the source's value for this sample
//!   source runs later, or is the node  → Previous: the source's value for the previous sample
//!     itself (back edge / self loop)                (default, i.e. 0.0, for sample 0)
//!   source is never evaluated          → Never:    always default
//! ```
//!
//! `Previous` edges are what make a graph recurrent: they carry state from one sample to the
//! next. But only the nodes *on a cycle* depend on their own past and have to step through
//! samples one at a time. Everything else can still be computed a whole column at a time:
//!
//! ```text
//!   x0 ──► w0 ──┐
//!               add ◄──┐            ┌──────────────┬──────────────┬───────────────┐
//!   x1 ──► w1 ──┘  │   │ Previous   │ column       │ row by row   │ column        │
//!                  ▼   │            │ x0 x1 w0 w1  │ add h        │ sigmoid       │
//!                  h ──┘ ──► sigmoid└──────────────┴──────────────┴───────────────┘
//! ```
//!
//! [GraphPlan::compile] finds the cycles as the graph's strongly connected components, and
//! orders all components so every node's inputs are finished before it runs:
//!
//! - [Step::Column] (a node not on a cycle): computed for every sample at once.
//! - [Step::Cycle] (a component with more than one node, or a self loop): its nodes step
//!   through the samples together, in the row evaluator's order.

use super::{GraphNode, cycles::Sccs, iter::GraphIterator};
use crate::NodeType;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::ops::Range;

/// Which sample's value an edge reads, following the row evaluator's rule (see the module docs).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum When {
    /// The source's value for the current sample.
    Current,
    /// The source's value for the previous sample (default for sample 0).
    Previous,
    /// The source is never evaluated, so it's always default.
    Never,
}

/// One input of a node: which slot it reads, and which sample's value.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Input {
    pub slot: u32,
    pub at: When,
}

/// One unit of the plan.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Step {
    /// A node not on a cycle, computed for every sample at once.
    Column(u32),
    /// The slots of one cycle, stepping through the samples together.
    Cycle(Range<u32>),
}

/// A graph compiled into a plan. Structure only: ops are read from the nodes when the plan
/// runs, so a plan stays valid when only node values change.
///
/// For a graph with a cycle between `add` and `h`:
///
/// ```text
///   node index:   0=x0  1=x1  2=add  3=h  4=sigmoid
///   edges:        x0→add, x1→add, h→add (back edge), add→h, h→sigmoid
///
///   row evaluator order (iter_topological):   x0 x1 add h sigmoid
///   h→add points backwards in that order, so add reads h's *previous* sample: Previous.
///
///   slot:       0          1          2      3     4
///   node_of = [ 0,         1,         2,     3,    4 ]          slot → node index (plan order)
///   steps   = [ Column(0), Column(1), Cycle(2..4), Column(4) ]  add and h step together
///
///   inputs (reading `input_ranges` together with `inputs`):
///     slot 2 add:     [ (0, Current), (1, Current), (3, Previous) ]
///     slot 3 h:       [ (2, Current) ]
///     slot 4 sigmoid: [ (3, Current) ]
/// ```
///
/// Slots are assigned in plan order, not the row evaluator's order, so every column step's
/// inputs are always in finished rows above it.
#[derive(Default, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GraphPlan {
    steps: Vec<Step>,
    node_of: Vec<usize>,
    slot_of: Vec<u32>,
    inputs: Vec<Input>,
    input_ranges: Vec<(u32, u32)>,
    output_slots: Vec<u32>,
    row_order: Vec<u32>,
    order: Vec<usize>,
    pos_of: Vec<u32>,
    cycles: Sccs,
}

impl GraphPlan {
    #[inline]
    pub fn steps(&self) -> &[Step] {
        &self.steps
    }

    #[inline]
    pub fn node_of(&self, slot: usize) -> usize {
        self.node_of[slot]
    }

    #[inline]
    pub fn slots(&self) -> usize {
        self.slot_of.len()
    }

    #[inline]
    pub fn inputs(&self, slot: usize) -> &[Input] {
        let (start, end) = self.input_ranges[slot];
        &self.inputs[start as usize..end as usize]
    }

    #[inline]
    pub fn output_slots(&self) -> &[u32] {
        &self.output_slots
    }

    /// Evaluated slots in the row evaluator's order ([GraphIterator::iter_topological]). Running
    /// them in this order, with each slot reading whatever its sources currently hold, evaluates
    /// one sample exactly like the row evaluator: a source that already ran gives this sample's
    /// value, one that hasn't yet gives the previous sample's.
    ///
    /// ```text
    ///   row_order = [ x0, x1, add, h, sigmoid ]   (as slots)
    ///   add reads h before h has run this sample → h's previous value (Previous)
    /// ```
    #[inline]
    pub fn row_order(&self) -> &[u32] {
        &self.row_order
    }

    /// True when the plan has at least one [Step::Cycle].
    pub fn is_cyclic(&self) -> bool {
        self.steps.iter().any(|step| matches!(step, Step::Cycle(_)))
    }

    /// Compiles `nodes` into this plan in place, reusing its allocations:
    ///
    /// 1. Take the row evaluator's order from [GraphIterator::iter_topological]. Its position for
    ///    each node decides each edge's [When]: `Current` if the source comes earlier, `Previous`
    ///    if it comes later or is the node itself, `Never` if it's never reached.
    /// 2. Find the strongly connected components over all edges (`Current` and `Previous`). A
    ///    component with more than one node, or a node with a self loop, is a cycle.
    /// 3. Lay the components out inputs first; that's the plan. Slots are assigned in plan order,
    ///    so everything a column step reads sits in the rows above it.
    ///
    /// ```text
    ///   feed-forward: every source earlier            each node is its own Step::Column
    ///     x0 ── w ── add ── sigmoid
    ///     x1 ───────┘
    ///
    ///   back edge: h → add points backwards           {add, h} is one Step::Cycle
    ///     x0 ── add ── h ── sigmoid
    ///           ▲      │
    ///           └──────┘
    /// ```
    pub fn compile<T>(&mut self, nodes: &[GraphNode<T>]) {
        let Self {
            order,
            pos_of,
            cycles,
            ..
        } = self;

        // 1. The row evaluator's order, and each node's position in it.
        order.clear();
        order.extend(nodes.iter_topological().map(|n| n.index()));

        pos_of.clear();
        pos_of.resize(nodes.len(), u32::MAX);
        for (pos, &index) in order.iter().enumerate() {
            pos_of[index] = pos as u32;
        }

        // 2. Strongly connected components over the evaluated nodes, by position. Unreached
        //    sources are constant, so they don't take part.
        cycles.edges.clear();
        cycles.edge_ranges.clear();
        for &index in order.iter() {
            let start = cycles.edges.len() as u32;
            for &src in nodes[index].incoming() {
                let pos = pos_of[src];
                if pos != u32::MAX {
                    cycles.edges.push(pos);
                }
            }
            cycles.edge_ranges.push((start, cycles.edges.len() as u32));
        }
        cycles.find(order.len());

        // 3. The plan: components in dependency order, each cycle's nodes in the row evaluator's
        //    order (the order they run within a sample).
        self.node_of.clear();
        self.steps.clear();
        for c in 0..cycles.component_ends.len() {
            let start = if c == 0 {
                0
            } else {
                cycles.component_ends[c - 1]
            } as usize;
            let end = cycles.component_ends[c] as usize;
            let members = &mut cycles.components[start..end];
            members.sort_unstable();

            let is_cycle = members.len() > 1 || {
                let p = members[0];
                let (es, ee) = cycles.edge_ranges[p as usize];
                cycles.edges[es as usize..ee as usize].contains(&p)
            };

            let first = self.node_of.len() as u32;
            self.node_of
                .extend(members.iter().map(|&pos| order[pos as usize]));
            self.steps.push(if is_cycle {
                Step::Cycle(first..self.node_of.len() as u32)
            } else {
                Step::Column(first)
            });
        }

        // Node index → slot. Nodes the iterator never yields (e.g. every node of an invalid graph)
        // get trailing slots after the evaluated ones; they're never written, so they read as
        // default, like the row evaluator.
        //
        //   slots:  [ 0 .. evaluated )                [ evaluated .. nodes.len() )
        //           plan order                        unreached, stay default
        self.slot_of.clear();
        self.slot_of.resize(nodes.len(), u32::MAX);
        for (slot, &index) in self.node_of.iter().enumerate() {
            self.slot_of[index] = slot as u32;
        }
        let unreached = self.slot_of.iter_mut().filter(|s| **s == u32::MAX);
        for (next, slot) in (self.node_of.len() as u32..).zip(unreached) {
            *slot = next;
        }

        // Each slot's inputs, with which sample each edge reads.
        self.inputs.clear();
        self.input_ranges.clear();
        for &index in &self.node_of {
            let start = self.inputs.len() as u32;
            let pos = pos_of[index];
            for &src in nodes[index].incoming() {
                let src_pos = pos_of[src];
                let at = if src_pos == u32::MAX {
                    When::Never
                } else if src_pos >= pos {
                    When::Previous
                } else {
                    When::Current
                };
                self.inputs.push(Input {
                    slot: self.slot_of[src],
                    at,
                });
            }
            self.input_ranges.push((start, self.inputs.len() as u32));
        }

        self.row_order.clear();
        self.row_order
            .extend(order.iter().map(|&index| self.slot_of[index]));

        self.output_slots.clear();
        self.output_slots.extend(
            nodes
                .get_nodes_of_type(NodeType::Output)
                .map(|n| self.slot_of[n.index()]),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Graph, Op};

    #[test]
    fn test_recompile_in_place() {
        let mut a = Graph::<Op<f32>>::default();
        let x = a.insert(NodeType::Input, Op::var(0));
        let o = a.insert(NodeType::Output, Op::linear());
        a.attach(x, o);

        let mut b = Graph::<Op<f32>>::default();
        let x = b.insert(NodeType::Input, Op::var(0));
        let v = b.insert(NodeType::Vertex, Op::sum());
        let o = b.insert(NodeType::Output, Op::linear());
        b.attach(x, v).attach(v, v).attach(v, o);

        let mut plan = GraphPlan::default();
        plan.compile(a.as_ref());
        assert!(!plan.is_cyclic());

        plan.compile(b.as_ref());
        assert!(plan.is_cyclic());
        assert_eq!(plan.output_slots(), &[2]);
        assert_eq!(plan.inputs(1)[1].at, When::Previous);
    }
}
