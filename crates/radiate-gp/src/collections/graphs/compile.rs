//! Compiling a graph into a [GraphPlan]: the order to run its nodes in, what each node reads,
//! and (for batches) which nodes can run a whole column at a time. Structure only;
//! [GraphEvaluator](super::GraphEvaluator) runs it.
//!
//! A plan has two layers, so one-sample-at-a-time evaluation never pays for the batch layer:
//!
//! ```text
//!   row plan          always built        order, input ranges, outputs      cheap
//!   column schedule   first batch only    edge timing, cycles, steps        a few × the row plan
//! ```
//!
//! Everything is indexed by node index: a node's state, its row in the batch value matrix,
//! and its edge range all use the index it has in the graph.
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
//! Evaluating one sample needs none of this: running the order and reading the sources'
//! current values *is* the rule. The column schedule spells it out per edge, because a batch
//! computes whole columns and has to know which sample each edge means.
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
//! [GraphPlan::schedule] finds the cycles as the graph's strongly connected components, and
//! orders all components so every node's inputs are finished before it runs:
//!
//! - [Step::Column] (a node not on a cycle): computed for every sample at once.
//! - [Step::Cycle] (a component with more than one node, or a self loop): its nodes step
//!   through the samples together, in the row evaluator's order.

use super::{GraphNode, NodeIdx, cycles::Sccs, iter::GraphIterator};
use crate::NodeType;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::ops::Range;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum When {
    Current,
    Previous,
}

/// One unit of the column schedule.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Step {
    Column(NodeIdx),
    Cycle(Range<usize>),
}

/// A graph compiled into a plan. Ops are read from the nodes when the plan
/// runs, so a plan stays valid when only node values change.
///
/// For a graph with a cycle between `add` and `h`:
///
/// ```text
///   node index:   0=x0  1=x1  2=add  3=h  4=sigmoid
///   edges:        x0→add, x1→add, h→add (back edge), add→h, h→sigmoid
///
///   row plan:
///     order          = [ 0, 1, 2, 3, 4 ]               iter_topological
///     input_ranges   = [ 0..0, 0..0, 0..3, 3..4, 4..5 ] each node's edges, in incoming() order
///     output_indices = [ 4 ]
///
///   column schedule:
///     when  = [ Current, Current, Previous,  Current,  Current ]
///               └──────── add's edges ────┘  └ h ──┘  └ sigmoid
///             h→add points backwards in the order, so add reads h's *previous* sample.
///     steps = [ Column(0), Column(1), Cycle(0..2), Column(4) ]
///     cycle_nodes = [ 2, 3 ]                            add and h step together
/// ```
#[derive(Default, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GraphPlan {
    // Row wise plan
    order: Vec<NodeIdx>,
    input_ranges: Vec<(u32, u32)>,
    output_indices: Vec<NodeIdx>,

    // Column schedule
    has_schedule: bool,
    when: Vec<When>,
    steps: Vec<Step>,
    cycle_nodes: Vec<NodeIdx>,
    pos_of: Vec<u32>,
    sccs: Sccs,
}

impl GraphPlan {
    /// Node indices in the row evaluator's order ([GraphIterator::iter_topological]). Running
    /// them in this order, with each node reading whatever its sources currently hold, evaluates
    /// one sample: a source that already ran gives this sample's value, one that hasn't yet
    /// gives the previous sample's.
    #[inline]
    pub fn order(&self) -> &[NodeIdx] {
        &self.order
    }

    /// Node `index`'s edges, as a range into an edge-indexed buffer of length [GraphPlan::edges].
    /// Edge `k` of the range is the node's `k`-th incoming edge.
    #[inline]
    pub fn inputs(&self, index: NodeIdx) -> Range<usize> {
        let (start, end) = self.input_ranges[index.as_usize()];
        start as usize..end as usize
    }

    /// Total number of edges in the graph.
    #[inline]
    pub fn edges(&self) -> usize {
        self.input_ranges.last().map_or(0, |&(_, end)| end as usize)
    }

    #[inline]
    pub fn output_indices(&self) -> &[NodeIdx] {
        &self.output_indices
    }

    #[inline]
    pub fn has_schedule(&self) -> bool {
        self.has_schedule
    }

    #[inline]
    pub fn when(&self, index: NodeIdx) -> &[When] {
        &self.when[self.inputs(index)]
    }

    #[inline]
    pub fn steps(&self) -> &[Step] {
        &self.steps
    }

    #[inline]
    pub fn cycle_nodes(&self, range: Range<usize>) -> &[NodeIdx] {
        &self.cycle_nodes[range]
    }

    pub fn is_cyclic(&self) -> bool {
        debug_assert!(self.has_schedule, "is_cyclic needs the column schedule");
        self.steps.iter().any(|step| matches!(step, Step::Cycle(_)))
    }

    /// Compiles the row plan for `nodes` in place, reusing its allocations, and drops any
    /// column schedule (it belonged to the previous graph if using in a buffered context).
    pub fn compile<T>(&mut self, nodes: &[GraphNode<T>]) {
        self.order.clear();
        self.order.reserve(nodes.len());
        self.order
            .extend(nodes.iter_topological().map(|n| n.index()));

        self.input_ranges.clear();
        self.input_ranges.reserve(nodes.len());
        let mut start = 0;
        self.input_ranges.extend(nodes.iter().map(|node| {
            let end = start + node.incoming().len() as u32;
            let range = (start, end);
            start = end;
            range
        }));

        self.output_indices.clear();
        self.output_indices
            .extend(nodes.get_nodes_of_type(NodeType::Output).map(|n| n.index()));

        self.has_schedule = false;
    }

    /// Builds the column schedule on top of the row plan, if it isn't built yet. `nodes` must
    /// be the graph the plan was compiled from.
    ///
    /// 1. Each node's position in the row order decides each edge's [When]: `Current` if the
    ///    source comes earlier, `Previous` if it comes later or is the node itself, `Never` if
    ///    it's never reached.
    /// 2. Find the strongly connected components over all edges (`Current` and `Previous`). A
    ///    component with more than one node, or a node with a self loop, is a cycle.
    /// 3. Lay the components out inputs first; that's the schedule.
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
    pub fn schedule<T>(&mut self, nodes: &[GraphNode<T>]) {
        if self.has_schedule {
            return;
        }

        let Self {
            order,
            pos_of,
            sccs,
            when,
            steps,
            cycle_nodes,
            ..
        } = self;

        // 1. Each node's position in the row order, and what each edge reads.
        pos_of.clear();
        pos_of.resize(nodes.len(), u32::MAX);
        for (pos, &index) in order.iter().enumerate() {
            pos_of[index.as_usize()] = pos as u32;
        }

        let edges = self.input_ranges.last().map_or(0, |&(_, e)| e as usize);

        when.clear();
        when.reserve(edges);
        let mut any_previous = false;
        for (index, node) in nodes.iter().enumerate() {
            let pos = pos_of[index];
            when.extend(node.incoming().iter().map(|&src| {
                let src_pos = pos_of[src.as_usize()];
                if src_pos >= pos {
                    any_previous = true;
                    When::Previous
                } else {
                    When::Current
                }
            }));
        }

        steps.clear();
        steps.reserve(order.len());
        cycle_nodes.clear();
        if !any_previous {
            // Every evaluated edge points earlier in the row order: no cycles, and the row
            // order is already inputs-first.
            steps.reserve(order.len());
            steps.extend(order.iter().map(|&i| Step::Column(i)));
            self.has_schedule = true;
            return;
        }

        // 2. Strongly connected components over the evaluated nodes, by position. Unreached
        //    sources are constant, so they don't take part.
        sccs.edges.clear();
        sccs.edges.reserve(nodes.len());
        sccs.edge_ranges.clear();
        sccs.edge_ranges.reserve(nodes.len());
        for &index in order.iter() {
            let start = sccs.edges.len() as u32;
            for &src in nodes[index.as_usize()].incoming() {
                let pos = pos_of[src.as_usize()];
                if pos != u32::MAX {
                    sccs.edges.push(pos);
                }
            }
            sccs.edge_ranges.push((start, sccs.edges.len() as u32));
        }
        sccs.find(order.len());

        // 3. The schedule: components in dependency order, each cycle's nodes in the row
        //    evaluator's order (the order they run within a sample).
        cycle_nodes.reserve(nodes.len());
        for c in 0..sccs.component_ends.len() {
            let start = if c == 0 {
                0
            } else {
                sccs.component_ends[c - 1]
            } as usize;
            let end = sccs.component_ends[c] as usize;
            let members = &mut sccs.components[start..end];
            members.sort_unstable();

            let is_cycle = members.len() > 1 || {
                let p = members[0];
                let (es, ee) = sccs.edge_ranges[p as usize];
                sccs.edges[es as usize..ee as usize].contains(&p)
            };

            if is_cycle {
                let first = cycle_nodes.len();
                cycle_nodes.extend(members.iter().map(|&pos| order[pos as usize]));
                steps.push(Step::Cycle(first..cycle_nodes.len()));
            } else {
                steps.push(Step::Column(order[members[0] as usize]));
            }
        }

        self.has_schedule = true;
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
        let o = a.insert(NodeType::Output, Op::sum());
        a.attach(x, o);

        let mut b = Graph::<Op<f32>>::default();
        let x = b.insert(NodeType::Input, Op::var(0));
        let v = b.insert(NodeType::Vertex, Op::sum());
        let o = b.insert(NodeType::Output, Op::sum());
        b.attach(x, v).attach(v, v).attach(v, o);

        let mut plan = GraphPlan::default();
        plan.compile(a.as_ref());
        plan.schedule(a.as_ref());
        assert!(!plan.is_cyclic());

        plan.compile(b.as_ref());
        assert!(!plan.has_schedule());
        assert_eq!(plan.edges(), 3);
        assert_eq!(plan.output_indices(), &[2]);

        plan.schedule(b.as_ref());
        assert!(plan.is_cyclic());
        assert_eq!(plan.when(NodeIdx::new(1)), &[When::Current, When::Previous]);
        assert_eq!(plan.steps()[1], Step::Cycle(0..1));
        assert_eq!(plan.cycle_nodes(0..1), &[1]);
    }
}
