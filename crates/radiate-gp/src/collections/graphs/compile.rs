//! PROTOTYPE: compiling a graph into a [GraphKernel], and evaluating it over a whole batch of
//! samples at once. Cyclic graphs included.
//!
//! # Rows vs. columns
//!
//! Take a small graph evaluated over a data set of `N` samples:
//!
//! ```text
//!   x0 ──► w(0.5) ──┐
//!                   add ──► sigmoid ──► (output)
//!   x1 ─────────────┘
//! ```
//!
//! The row evaluator ([GraphEvaluator](super::GraphEvaluator)) walks the graph once per sample.
//! Every node of every sample is a separate `op.eval` call through a function pointer:
//!
//! ```text
//!   for sample in 0..N:                      N × nodes indirect calls
//!       x0 → w → x1 → add → sigmoid          (each call: gather inputs, call, return)
//! ```
//!
//! [GraphKernel::eval_batch] flips the loops. Each node runs once over *all* samples,
//! producing a whole column of values, and its kernel is a plain loop the compiler can inline
//! and vectorize:
//!
//! ```text
//!   for node in [x0, w, x1, add, sigmoid]:   `nodes` dispatches total
//!       out[0..N] = kernel(input columns)    (one tight loop over N samples)
//! ```
//!
//! # Compiled graph vs. batch state
//!
//! A [GraphKernel] is *what* to compute and in what order. It depends only on the graph's
//! structure: no ops (they're read from the nodes at evaluation time), no data, no state. A
//! [BatchState] is *where* values live while a batch is evaluated, and is sized by the batch.
//! Both are reusable: recompile and re-evaluate in place without reallocating.
//!
//! # Memory layout
//!
//! Every node gets a *slot*: its row in a `slots × samples` [Matrix]. Row `s` holds slot `s`'s
//! value for every sample, so reading or writing a node's column is one contiguous slice.
//!
//! ```text
//!                 sample →   0      1      2     ...   N-1
//!   slot 0  x0           [ 5.1    4.9    6.3   ...   5.8 ]   ◄─ copied from features row 0
//!   slot 1  w(0.5)       [ 2.55   2.45   3.15  ...   2.9 ]   ◄─ reads slot 0
//!   slot 2  x1           [ 3.5    3.0    3.3   ...   2.7 ]   ◄─ copied from features row 1
//!   slot 3  add          [ 6.05   5.45   6.45  ...   5.6 ]   ◄─ reads slots 1, 2
//!   slot 4  sigmoid      [ 0.99   0.99   0.99  ...   0.99]   ◄─ reads slot 3  (output)
//! ```
//!
//! Inputs come from the data set's features *transposed* to `features × samples`, so input
//! node `x_i`'s column is just row `i` of that matrix. A `DataSet` stores samples row-major
//! (`samples × features`, one row per sample); `Regression` transposes it once, lazily:
//!
//! ```text
//!   DataSet features (samples × features)        transposed (features × samples)
//!            x0   x1   x2   x3                               sample →  0    1    2   ...
//!   s0    [ 5.1  3.5  1.4  0.2 ]                 row 0 (x0) [ 5.1  4.9  6.3  ... ]
//!   s1    [ 4.9  3.0  1.4  0.2 ]       ──►       row 1 (x1) [ 3.5  3.0  3.3  ... ]
//!   s2    [ 6.3  3.3  6.0  2.5 ]                 row 2 (x2) [ 1.4  1.4  6.0  ... ]
//!   ...                                          row 3 (x3) [ 0.2  0.2  2.5  ... ]
//! ```
//!
//! # Cycles
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
//! [GraphKernel::compile] finds the cycles as the graph's strongly connected components
//! (Tarjan's algorithm), and orders all components so every node's inputs are finished before
//! it runs:
//!
//! - [Step::Column] (a node not on a cycle): computed for every sample at once. A `Previous`
//!   input is its source's column lagged one sample, with default at the top.
//! - [Step::Cycle] (a component with more than one node, or a self loop): its nodes step through
//!   the samples together, in the row evaluator's order, reading `Previous` inputs from the
//!   previous sample's entry. Inputs from outside the cycle are already complete columns.
//!
//! Every node computes exactly what the row evaluator computes, from exactly the same inputs,
//! so results are bit-identical, recurrent graphs included.
//!
//! # Buffers at a glance
//!
//! Shapes for iris: 113 samples, 4 features, 3 label columns, a graph of ~40 nodes.
//!
//! ```text
//!   buffer          shape                   holds                          lifetime
//!   ──────────────  ──────────────────────  ─────────────────────────────  ──────────────────────
//!   features        4 × 113                 transposed input data          once per Regression
//!
//!   GraphKernel                                                          rebuilt per graph,
//!   node_of         [evaluated nodes]       slot → node index                allocations reused
//!   slot_of         [nodes]                 node index → slot
//!   inputs          [total edges]           (source slot, When)
//!   input_ranges    [evaluated nodes]       slot → range into inputs
//!   steps           [components]            Column(slot) or Cycle(slots)
//!   output_slots    [outputs]               slots of the output nodes
//!
//!   BatchState                                                             reset per graph,
//!   values          nodes × 113             every node's values              allocations reused
//!     prev / out    views into values       finished slots / current slot  free (split_row_mut)
//!   lagged          [Previous inputs × 113] Previous input columns, lagged column steps only
//!   defaults        113                     a default column (Never reads)
//!   scratch         [fan-in]                one sample's inputs            fallback + cycles
//!
//!   columns         [&[F]; 8]               pointers to input columns      stack, column steps
//!   labels          113 × 3                 targets (DataSet's own)        untouched
//! ```
//!
//! On iris `values` is ~40 × 113 × 4 bytes ≈ 18 KB, small enough to stay in L1/L2. One
//! [GraphKernel] and one [BatchState] are created per `evaluate` batch and reused for every
//! graph in it, so after the first graph almost nothing is allocated.
//!
//! See `graph.md` for the investigation and benchmark numbers.

use super::{GraphNode, iter::GraphIterator};
use crate::{
    Eval, NodeType, Op,
    node::Node,
    ops::{OpFloat, math::column_kernel},
};
use radiate_utils::Matrix;
use std::ops::Range;

/// How each [Step::Column] is computed. Both modes use the same column layout; they exist so
/// the gain from the layout and the gain from kernels can be measured separately.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BatchMode {
    /// Column layout, but every node calls `op.eval` once per sample.
    Generic,
    /// Column layout with per-op kernels for the common ops; others fall back to [BatchMode::Generic].
    Kernels,
}

/// Counters for benchmarking: how many graphs had cycles, and how many nodes ran a kernel, the
/// per-sample fallback, or inside a cycle step.
#[derive(Default, Debug)]
pub struct BatchStats {
    pub graphs: usize,
    pub cyclic_graphs: usize,
    pub kernel_nodes: usize,
    pub fallback_nodes: usize,
    pub cycle_nodes: usize,
}

/// Which sample's value an edge reads, following the row evaluator's rule (see the module docs).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum When {
    /// The source's value for the current sample.
    Current,
    /// The source's value for the previous sample (default for sample 0).
    Previous,
    /// The source is never evaluated, so it's always default.
    Never,
}

/// One input of a node: which slot it reads, and which sample's value.
#[derive(Clone, Copy, Debug)]
pub struct Input {
    pub slot: u32,
    pub at: When,
}

/// One unit of the plan.
#[derive(Clone, Debug)]
pub enum Step {
    /// A node not on a cycle, computed for every sample at once.
    Column(u32),
    /// The slots of one cycle, stepping through the samples together.
    Cycle(Range<u32>),
}

/// A graph compiled into a plan: which slot each node lives in, what each slot reads, and the
/// order to run them in. Structure only: ops are read from the nodes at evaluation time, so a
/// compiled graph stays valid when only node values change.
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
#[derive(Default, Debug)]
pub struct GraphKernel {
    /// The plan: every component of the graph, inputs first.
    steps: Vec<Step>,
    /// Slot → node index, in plan order.
    node_of: Vec<usize>,
    /// Node index → slot. Nodes never reached get trailing slots that are never written.
    slot_of: Vec<u32>,
    /// Every slot's inputs, flattened. Slot `s` reads `inputs[input_ranges[s]]`, in the node's
    /// incoming order. An empty range means the node reads the sample's features.
    inputs: Vec<Input>,
    /// Slot → range into `inputs`.
    input_ranges: Vec<(u32, u32)>,
    /// Slots of the graph's output nodes, in output order.
    output_slots: Vec<u32>,
    /// Scratch used while compiling.
    scratch: CompileScratch,
}

/// Compile-time scratch, kept so recompiling doesn't reallocate.
#[derive(Default, Debug)]
struct CompileScratch {
    /// Position → node index, in the row evaluator's order.
    order: Vec<usize>,
    /// Node index → position in `order` (`u32::MAX` if never reached).
    pos_of: Vec<u32>,
    cycles: CycleFinder,
}

/// Where values live while a [GraphKernel] is evaluated over a batch of samples.
#[derive(Default)]
pub struct BatchState<F> {
    /// Every slot's values, shape `nodes × samples`: row `s` is slot `s` for every sample. It has
    /// one row per *node* (not just per evaluated slot), so unreached nodes' trailing slots exist
    /// and read as default. Underneath it's one flat `Vec`, slot after slot, and slot `s` /
    /// sample `r` lives at `s * samples + r`:
    ///
    /// ```text
    ///   index:  0 ..... 112 | 113 .... 225 | 226 .... 338 | 339 .... 451 | 452 .... 564
    ///           ── slot 0 ── ── slot 1 ──── ── slot 2 ──── ── slot 3 ──── ── slot 4 ────
    /// ```
    values: Matrix<F>,
    /// `Previous` input columns for the current column step, lagged one sample:
    ///
    /// ```text
    ///   source column:   [ v0  v1  v2  ...  v(N-1) ]
    ///   lagged:          [ 0   v0  v1  ...  v(N-2) ]   (sample r reads the source at r-1)
    /// ```
    lagged: Vec<F>,
    /// A column of defaults, the input for `Never` reads.
    defaults: Vec<F>,
    /// One sample's inputs, gathered whenever an op has to be called once per sample with a
    /// `&[F]`: the column fallback (ops without a kernel) and cycle steps. Width is the node's
    /// fan-in, or the feature count for a node with no inputs (it gets that sample's whole
    /// feature row). Resized on demand and reused.
    ///
    /// ```text
    ///   node with inputs = [(1, Current), (3, Previous)], at sample r:
    ///     scratch = [ values[1][r], values[3][r-1] ]  →  out[r] = op.eval(scratch)
    /// ```
    scratch: Vec<F>,
}

impl<F> BatchState<F> {
    /// Every slot's values from the last batch evaluated: one row per slot, one column per sample.
    #[inline]
    pub fn values(&self) -> &Matrix<F> {
        &self.values
    }
}

impl GraphKernel {
    /// Compiles `nodes` into a new [GraphKernel].
    pub fn new<T>(nodes: &[GraphNode<T>]) -> Self {
        let mut compiled = Self::default();
        compiled.compile(nodes);
        compiled
    }

    /// Slots of the graph's output nodes, in output-node order. Slot `s` is row `s` of
    /// [BatchState::values].
    #[inline]
    pub fn output_slots(&self) -> &[u32] {
        &self.output_slots
    }

    /// True when the plan has at least one [Step::Cycle].
    pub fn is_cyclic(&self) -> bool {
        self.steps.iter().any(|step| matches!(step, Step::Cycle(_)))
    }

    /// Recompiles in place for `nodes`, reusing this compiled graph's allocations:
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
        let CompileScratch {
            order,
            pos_of,
            cycles,
        } = &mut self.scratch;

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

        self.output_slots.clear();
        self.output_slots.extend(
            nodes
                .get_nodes_of_type(NodeType::Output)
                .map(|n| self.slot_of[n.index()]),
        );
    }

    #[inline]
    fn inputs_of(&self, slot: usize) -> &[Input] {
        let (start, end) = self.input_ranges[slot];
        &self.inputs[start as usize..end as usize]
    }

    /// Evaluates the compiled graph over every sample. `nodes` must be the nodes it was compiled
    /// from (only their ops may have changed), and `features` is the data set's features
    /// transposed: one row per feature, one column per sample
    /// (`DataSet::feature_matrix().transpose()`).
    ///
    /// Results are bit-identical to running the row evaluator over the samples in order from a
    /// fresh state. Read them from [BatchState::values] at [GraphKernel::output_slots].
    pub fn eval_batch<F: OpFloat>(
        &self,
        nodes: &[GraphNode<Op<F>>],
        features: &Matrix<F>,
        state: &mut BatchState<F>,
        mode: BatchMode,
        stats: &mut BatchStats,
    ) {
        debug_assert_eq!(
            self.slot_of.len(),
            nodes.len(),
            "compiled for a different graph"
        );
        stats.graphs += 1;

        let n_samples = features.cols();
        // One row per node (not just per evaluated slot), zeroed, so unreached trailing slots read
        // as default exactly like the row evaluator's untouched outputs.
        state
            .values
            .reshape_and_fill(nodes.len(), n_samples, F::default());
        state.defaults.clear();
        state.defaults.resize(n_samples, F::default());

        if self.is_cyclic() {
            stats.cyclic_graphs += 1;
        }

        for step in &self.steps {
            match step {
                Step::Column(slot) => {
                    self.eval_column(nodes, features, state, *slot as usize, mode, stats)
                }
                Step::Cycle(slots) => {
                    stats.cycle_nodes += slots.len();
                    self.eval_cycle(nodes, features, state, slots.clone());
                }
            }
        }
    }

    /// Computes one node's whole column (a node not on a cycle).
    fn eval_column<F: OpFloat>(
        &self,
        nodes: &[GraphNode<Op<F>>],
        features: &Matrix<F>,
        state: &mut BatchState<F>,
        slot: usize,
        mode: BatchMode,
        stats: &mut BatchStats,
    ) {
        let n = features.cols();
        let BatchState {
            values,
            lagged,
            defaults,
            scratch,
        } = state;

        let op = nodes[self.node_of[slot]].value();
        let inputs = self.inputs_of(slot);

        // Split `values` at this slot's row. Rust won't let us read and write `values` at the
        // same time, so `split_row_mut` hands back two non-overlapping views of the same memory,
        // at no cost: the finished rows as a read-only `MatrixView`, and this slot's row mutably.
        //
        //   computing slot 3 (add) in the module-docs example:
        //
        //   values.data():  [ slot 0 | slot 1 | slot 2 ][ slot 3 | slot 4 ]
        //                   └────────── prev ──────────┘└ out ─┘
        //                    MatrixView (slot rows × N)   &mut [F], this slot's N samples
        //
        // Slots are assigned in plan order, so every source this node reads is in `prev`.
        let (prev, out) = values.split_row_mut(slot);

        // `Previous` inputs read their source one sample back: copy each source column lagged
        // by one, with default for sample 0.
        let n_lagged = inputs.iter().filter(|i| i.at == When::Previous).count();
        lagged.clear();
        lagged.resize(n_lagged * n, F::default());
        for (k, input) in inputs.iter().filter(|i| i.at == When::Previous).enumerate() {
            if n > 0 {
                lagged[k * n + 1..(k + 1) * n]
                    .copy_from_slice(&prev.row(input.slot as usize)[..n - 1]);
            }
        }

        // Gather the input columns: slices straight into `prev`, `lagged` or `defaults`, nothing
        // copied. Up to `STACK_INPUTS` live on the stack; wider nodes spill to a `Vec`.
        let mut stack: [&[F]; STACK_INPUTS] = [&[]; STACK_INPUTS];
        let mut heap: Vec<&[F]> = Vec::new();
        let wide = inputs.len() > STACK_INPUTS;
        let mut k = 0;
        for (j, input) in inputs.iter().enumerate() {
            let column = match input.at {
                When::Current => prev.row(input.slot as usize),
                When::Previous => {
                    k += 1;
                    &lagged[(k - 1) * n..k * n]
                }
                When::Never => defaults.as_slice(),
            };
            if wide {
                heap.push(column);
            } else {
                stack[j] = column;
            }
        }
        let columns: &[&[F]] = if wide { &heap } else { &stack[..inputs.len()] };

        if mode == BatchMode::Kernels && kernel(op, columns, features, out) {
            stats.kernel_nodes += 1;
            return;
        }
        stats.fallback_nodes += 1;

        // Fallback: no kernel for this op, so call it once per sample like the row evaluator,
        // gathering that sample's inputs out of the columns into `scratch`:
        //
        //   sample r:   scratch = [ columns[0][r], columns[1][r], ... ]  →  out[r] = op.eval(scratch)
        //
        // Still one indirect call per sample, but always the same op back to back, which the
        // CPU predicts far better than the row evaluator's mix of ops.
        //
        // No incoming edges means the op reads the sample's features directly.
        let width = if columns.is_empty() {
            features.rows()
        } else {
            columns.len()
        };
        if scratch.len() < width {
            scratch.resize(width, F::default());
        }
        let scratch = &mut scratch[..width];

        for (r, o) in out.iter_mut().enumerate() {
            if columns.is_empty() {
                for (i, d) in scratch.iter_mut().enumerate() {
                    *d = features[(i, r)];
                }
            } else {
                for (d, column) in scratch.iter_mut().zip(columns) {
                    *d = column[r];
                }
            }
            *o = op.eval(scratch);
        }
    }

    /// Steps a cycle's nodes through the samples together, exactly like the row evaluator but
    /// restricted to the cycle:
    ///
    /// ```text
    ///   for sample r in 0..N:
    ///       for node in cycle (row evaluator order):
    ///           Current input  → values[src][r]      (earlier this sample, or a finished column)
    ///           Previous input → values[src][r - 1]  (default at r = 0)
    ///           values[node][r] = op.eval(inputs)
    /// ```
    ///
    /// Inputs from outside the cycle are complete columns by the time this runs, so the only
    /// sequential work is the cycle itself.
    fn eval_cycle<F: OpFloat>(
        &self,
        nodes: &[GraphNode<Op<F>>],
        features: &Matrix<F>,
        state: &mut BatchState<F>,
        slots: Range<u32>,
    ) {
        let BatchState {
            values, scratch, ..
        } = state;

        for r in 0..features.cols() {
            for slot in slots.start as usize..slots.end as usize {
                let op = nodes[self.node_of[slot]].value();
                let inputs = self.inputs_of(slot);

                let width = if inputs.is_empty() {
                    features.rows()
                } else {
                    inputs.len()
                };
                if scratch.len() < width {
                    scratch.resize(width, F::default());
                }
                let scratch = &mut scratch[..width];

                if inputs.is_empty() {
                    for (i, d) in scratch.iter_mut().enumerate() {
                        *d = features[(i, r)];
                    }
                } else {
                    for (d, input) in scratch.iter_mut().zip(inputs) {
                        *d = match input.at {
                            When::Current => values[(input.slot as usize, r)],
                            When::Previous if r > 0 => values[(input.slot as usize, r - 1)],
                            When::Previous | When::Never => F::default(),
                        };
                    }
                }

                values[(slot, r)] = op.eval(scratch);
            }
        }
    }
}

/// Strongly connected components, by Tarjan's algorithm (iterative, so deep graphs can't
/// overflow the stack). Edges point from a node to the nodes it reads, so a component is
/// emitted only after every component it reads from: `components` comes out inputs first.
#[derive(Default, Debug)]
struct CycleFinder {
    /// Each node's sources, flattened; node `v` reads `edges[edge_ranges[v].0..edge_ranges[v].1]`.
    edges: Vec<u32>,
    edge_ranges: Vec<(u32, u32)>,
    /// Output: every node, grouped by component; component `c` ends at `component_ends[c]`.
    components: Vec<u32>,
    component_ends: Vec<u32>,
    index: Vec<u32>,
    low: Vec<u32>,
    on_stack: Vec<bool>,
    stack: Vec<u32>,
    /// Explicit call stack: (node, next edge to visit).
    calls: Vec<(u32, u32)>,
}

impl CycleFinder {
    const UNVISITED: u32 = u32::MAX;

    fn find(&mut self, n: usize) {
        self.index.clear();
        self.index.resize(n, Self::UNVISITED);
        self.low.clear();
        self.low.resize(n, 0);
        self.on_stack.clear();
        self.on_stack.resize(n, false);
        self.stack.clear();
        self.calls.clear();
        self.components.clear();
        self.component_ends.clear();

        let mut next = 0;
        for root in 0..n as u32 {
            if self.index[root as usize] != Self::UNVISITED {
                continue;
            }

            self.visit(root, &mut next);
            while let Some(&(v, edge)) = self.calls.last() {
                let (_, end) = self.edge_ranges[v as usize];
                if edge < end {
                    self.calls.last_mut().unwrap().1 += 1;
                    let w = self.edges[edge as usize];
                    if self.index[w as usize] == Self::UNVISITED {
                        self.visit(w, &mut next);
                    } else if self.on_stack[w as usize] {
                        self.low[v as usize] = self.low[v as usize].min(self.index[w as usize]);
                    }
                    continue;
                }

                self.calls.pop();
                if let Some(&(parent, _)) = self.calls.last() {
                    self.low[parent as usize] = self.low[parent as usize].min(self.low[v as usize]);
                }

                if self.low[v as usize] == self.index[v as usize] {
                    loop {
                        let w = self.stack.pop().unwrap();
                        self.on_stack[w as usize] = false;
                        self.components.push(w);
                        if w == v {
                            break;
                        }
                    }
                    self.component_ends.push(self.components.len() as u32);
                }
            }
        }
    }

    fn visit(&mut self, v: u32, next: &mut u32) {
        self.index[v as usize] = *next;
        self.low[v as usize] = *next;
        *next += 1;
        self.stack.push(v);
        self.on_stack[v as usize] = true;
        self.calls.push((v, self.edge_ranges[v as usize].0));
    }
}

/// Input columns are collected into a stack array up to this fan-in, so the common case doesn't
/// allocate per node; wider nodes spill to a `Vec`.
const STACK_INPUTS: usize = 8;

/// Column kernel for `op`, or `false` if there isn't one (caller falls back to per-sample eval).
///
/// Inputs and constants are handled here since they depend on the data layout. Everything else
/// is handed to the built-in ops' own column kernels in `ops/math.rs` as a list of input columns:
///
/// ```text
///   inputs = [1, 2]   ──►   columns = [ prev.row(1), prev.row(2) ]   ──►   column_kernel(op, columns, out)
///                                       &[F; N]      &[F; N]                 e.g. add:
///                                                                            out[r] = add1(in0[r], in1[r])
/// ```
///
/// Nothing is copied: each column is a slice pointing straight into `prev`'s memory
/// (`prev.row(1)` is slot 1's 113 samples), a lagged `Previous` column, or the default column,
/// and the array holding them is just pointers and lengths on the stack.
fn kernel<F: OpFloat>(op: &Op<F>, columns: &[&[F]], features: &Matrix<F>, out: &mut [F]) -> bool {
    match op {
        // Input node: its column *is* row `index` of the transposed features.
        Op::Var(_, index, _) if columns.is_empty() => {
            out.copy_from_slice(features.row(*index));
            true
        }
        Op::Const(_, v) => {
            out.fill(*v);
            true
        }
        _ => column_kernel(op, columns, out),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Graph, GraphEvaluator, NodeStore, eval::EvalMut};
    use radiate_core::random_provider;

    /// Runs `graph` through the row evaluator (fresh state, samples in order) and through
    /// [GraphKernel::eval_batch] in both modes, and requires every output to match bit-for-bit.
    fn check(graph: &Graph<Op<f32>>, rows: &[Vec<f32>]) -> BatchStats {
        let features = Matrix::from(rows.to_vec()).transpose();
        let mut evaluator = GraphEvaluator::new(graph);
        let expected: Vec<Vec<f32>> = rows.iter().map(|r| evaluator.eval_mut(r)).collect();

        let compiled = GraphKernel::new(graph.as_ref());
        let mut stats = BatchStats::default();
        for mode in [BatchMode::Generic, BatchMode::Kernels] {
            let mut state = BatchState::default();
            compiled.eval_batch(graph.as_ref(), &features, &mut state, mode, &mut stats);
            for (r, want) in expected.iter().enumerate() {
                let got: Vec<u32> = compiled
                    .output_slots()
                    .iter()
                    .map(|&s| state.values()[(s as usize, r)].to_bits())
                    .collect();
                let want: Vec<u32> = want.iter().map(|v| v.to_bits()).collect();
                assert_eq!(got, want, "mode {mode:?}, row {r}");
            }
        }

        stats
    }

    fn rows(n: usize, width: usize) -> Vec<Vec<f32>> {
        (0..n)
            .map(|i| {
                (0..width)
                    .map(|j| ((i * 7 + j * 3) % 11) as f32 * 0.37 - 1.5)
                    .collect()
            })
            .collect()
    }

    #[test]
    fn test_batch_matches_row_evaluator() {
        let mut graph = Graph::<Op<f32>>::default();
        let a = graph.insert(NodeType::Input, Op::var(0));
        let b = graph.insert(NodeType::Input, Op::var(1));
        let w = graph.insert(NodeType::Edge, Op::weight_with(0.5));
        let add = graph.insert(NodeType::Vertex, Op::add());
        let exp = graph.insert(NodeType::Vertex, Op::exp());
        let out = graph.insert(NodeType::Output, Op::sigmoid());
        let out2 = graph.insert(NodeType::Output, Op::linear());

        graph.attach(a, w).attach(w, add).attach(b, add);
        graph.attach(b, exp).attach(add, out).attach(exp, out);
        graph.attach(a, out2).attach(add, out2);

        let stats = check(&graph, &rows(17, 2));
        assert_eq!(stats.cyclic_graphs, 0);
    }

    #[test]
    fn test_self_loop_matches_row_evaluator() {
        let mut graph = Graph::<Op<f32>>::default();
        let a = graph.insert(NodeType::Input, Op::var(0));
        let v = graph.insert(NodeType::Vertex, Op::sum());
        let out = graph.insert(NodeType::Output, Op::linear());
        graph.attach(a, v).attach(v, v).attach(v, out);

        let stats = check(&graph, &rows(9, 1));
        assert_eq!(stats.cyclic_graphs, 2);
        assert_eq!(stats.cycle_nodes, 2);
    }

    #[test]
    fn test_back_edge_cycle_matches_row_evaluator() {
        let mut graph = Graph::<Op<f32>>::default();
        let x0 = graph.insert(NodeType::Input, Op::var(0));
        let x1 = graph.insert(NodeType::Input, Op::var(1));
        let w0 = graph.insert(NodeType::Edge, Op::weight_with(0.8));
        let add = graph.insert(NodeType::Vertex, Op::sum());
        let h = graph.insert(NodeType::Vertex, Op::tanh());
        let out = graph.insert(NodeType::Output, Op::sigmoid());

        graph.attach(x0, w0).attach(w0, add).attach(x1, add);
        graph.attach(add, h).attach(h, add).attach(h, out);
        // Mark nodes on cycles as recurrent, like the graph builders and mutators do; without
        // it the topological iterator never schedules add/h at all.
        graph.set_cycles(vec![]);

        let stats = check(&graph, &rows(23, 2));
        assert_eq!(stats.cyclic_graphs, 2);
    }

    #[test]
    fn test_builder_graphs_match_row_evaluator() {
        random_provider::seed(42);
        let store = NodeStore::from(vec![
            (NodeType::Input, vec![Op::var(0), Op::var(1), Op::var(2)]),
            (NodeType::Edge, vec![Op::weight(), Op::identity()]),
            (NodeType::Vertex, crate::ops::all_ops()),
            (NodeType::Output, vec![Op::sigmoid(), Op::linear()]),
        ]);

        let graphs = [
            Graph::directed(3, 2, &store),
            Graph::recurrent(3, 2, &store),
            Graph::weighted_recurrent(3, 2, &store),
            Graph::lstm(3, 2, &store),
            Graph::gru(3, 2, &store),
        ];

        for (i, graph) in graphs.iter().enumerate() {
            let stats = check(graph, &rows(31, 3));
            // Every builder but `directed` produces a graph with at least one cycle step.
            assert_eq!(stats.cyclic_graphs > 0, i > 0, "graph {i}");
            assert!(stats.cycle_nodes > 0 || i == 0, "graph {i}");
        }
    }

    #[test]
    fn test_invalid_graph_reads_default() {
        // An output with no inputs is invalid, so nothing is evaluated and every output reads 0.
        let mut graph = Graph::<Op<f32>>::default();
        graph.insert(NodeType::Input, Op::var(0));
        graph.insert(NodeType::Output, Op::linear());

        check(&graph, &rows(5, 1));
    }

    #[test]
    fn test_recompile_in_place() {
        // A compiled graph can be recompiled for a different graph, reusing its allocations.
        let mut a = Graph::<Op<f32>>::default();
        let x = a.insert(NodeType::Input, Op::var(0));
        let o = a.insert(NodeType::Output, Op::linear());
        a.attach(x, o);

        let mut b = Graph::<Op<f32>>::default();
        let x = b.insert(NodeType::Input, Op::var(0));
        let v = b.insert(NodeType::Vertex, Op::sum());
        let o = b.insert(NodeType::Output, Op::linear());
        b.attach(x, v).attach(v, v).attach(v, o);

        let mut compiled = GraphKernel::new(a.as_ref());
        assert!(!compiled.is_cyclic());
        compiled.compile(b.as_ref());
        assert!(compiled.is_cyclic());
        assert_eq!(compiled.output_slots(), &[2]);
    }
}
