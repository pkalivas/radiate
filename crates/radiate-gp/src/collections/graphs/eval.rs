//! PROTOTYPE: [GraphKernel] evaluates a graph over a whole batch of samples at once, cyclic
//! graphs included, with results bit-identical to running the row evaluator sample by sample.
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
//! [GraphKernel] flips the loops. Each node runs once over *all* samples, producing a whole
//! column of values. Ops with a column kernel run it as a plain loop the compiler can inline and
//! vectorize; the rest are called once per sample (see the column `EvalInto` for `Op`):
//!
//! ```text
//!   for node in [x0, w, x1, add, sigmoid]:   `nodes` dispatches total
//!       out[0..N] = node.eval_into(columns)  (one tight loop over N samples)
//! ```
//!
//! Which nodes can run a column at a time, and which have to step through samples because
//! they're on a cycle, is decided by the [GraphPlan] (see [compile](super::compile)).
//!
//! # Memory layout
//!
//! Every node has a slot, and every slot a row in a `slots × samples` [Matrix]. Row `s` holds
//! slot `s`'s value for every sample, so reading or writing a node's column is one contiguous
//! slice.
//!
//! ```text
//!                 sample →   0      1      2     ...   N-1
//!   slot 0  x0           [ 5.1    4.9    6.3   ...   5.8 ]   ◄─ reads features row 0
//!   slot 1  w(0.5)       [ 2.55   2.45   3.15  ...   2.9 ]   ◄─ reads slot 0
//!   slot 2  x1           [ 3.5    3.0    3.3   ...   2.7 ]   ◄─ reads features row 1
//!   slot 3  add          [ 6.05   5.45   6.45  ...   5.6 ]   ◄─ reads slots 1, 2
//!   slot 4  sigmoid      [ 0.99   0.99   0.99  ...   0.99]   ◄─ reads slot 3  (output)
//! ```
//!
//! The kernel's input is the data set's features *transposed* to `features × samples`, so
//! feature `i` is row `i`. A node with no incoming edges gets every feature row as its input
//! columns, just as in the row evaluator it gets the sample's whole feature row, so an input
//! node `x_i` (`Op::var(i)`) reads column `i`. A `DataSet` stores samples row-major
//! (`samples × features`); `Regression` transposes it once, lazily:
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
//! The output is `outputs × samples`: row `j` is output node `j` for every sample.
//!
//! # Buffers at a glance
//!
//! Shapes for iris: 113 samples, 4 features, 4 label columns, a graph of ~40 nodes.
//!
//! ```text
//!   buffer          shape                   holds                          lifetime
//!   ──────────────  ──────────────────────  ─────────────────────────────  ──────────────────────
//!   features        4 × 113                 transposed input data          once per Regression
//!   outputs         outputs × 113           the output nodes' rows         per eval, reused
//!
//!   KernelCache                                                            reused across graphs
//!   plan            GraphPlan               slots, steps, inputs           recompiled per graph
//!   values          nodes × 113             every node's values            reset per eval
//!     prev / out    views into values       finished slots / current slot  free (split_row_mut)
//!   lagged          [Previous inputs × 113] Previous input columns, lagged column steps only
//!   defaults        113                     a default column (Never reads)
//!   scratch         [fan-in]                one sample's inputs            cycle steps only
//!
//!   columns         [&[V]; 8]               pointers to input columns      stack, column steps
//! ```
//!
//! On iris `values` is ~40 × 113 × 4 bytes ≈ 18 KB, small enough to stay in L1/L2. Carrying one
//! [KernelCache] from graph to graph ([GraphKernel::take_cache]) means almost nothing is
//! allocated after the first graph.
//!
//! See `graph.md` for the investigation and benchmark numbers.

use super::{
    GraphNode,
    compile::{GraphPlan, Step, When},
};
use crate::{Eval, EvalInto, EvalIntoMut, EvalMut, Graph, Node};
use radiate_utils::Matrix;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::ops::Range;

/// Input columns are collected into a stack array up to this fan-in, so the common case doesn't
/// allocate per node; wider nodes spill to a `Vec`.
const STACK_INPUTS: usize = 8;

/// A [GraphKernel]'s reusable state: the compiled plan and the buffers it runs in. Keep one
/// around across graphs ([GraphKernel::take_cache], `GraphKernel::from((&graph, cache))`) to
/// avoid reallocating.
#[derive(Default, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct KernelCache<V> {
    plan: GraphPlan,
    values: Matrix<V>,
    lagged: Vec<V>,
    defaults: Vec<V>,
    scratch: Vec<V>,
    row_values: Vec<V>,
    is_ready: bool,
}

impl<V> KernelCache<V>
where
    V: Default + Clone,
{
    pub fn new<T>(nodes: &[GraphNode<T>]) -> Self {
        let mut cache = KernelCache::default();
        cache.plan.compile(nodes);
        cache.row_values.resize(cache.plan.slots(), V::default());
        cache
    }

    pub fn slots(&self) -> usize {
        self.plan.slots()
    }

    pub fn bump(&mut self) {
        self.is_ready = false;
    }
}

/// [GraphKernel] compiles a graph into a [GraphPlan] on creation and evaluates it through
/// [EvalIntoMut] / [EvalMut], two ways:
///
/// - **One sample at a time**, `[V]` in, `[V]` out, like [GraphEvaluator](super::GraphEvaluator):
///   runs the plan in the row evaluator's order, keeping each node's last value between calls,
///   so recurrent graphs carry state from one sample to the next.
/// - **A whole batch at once**, `features × samples` in, `outputs × samples` out: runs the plan's
///   steps, a column at a time wherever the graph allows it.
///
/// Both give bit-identical outputs to feeding the same samples, in order, to a fresh
/// `GraphEvaluator` one at a time.
///
/// ```text
///   let mut kernel = GraphKernel::new(&graph);
///   let output = kernel.eval_mut(&sample);        // sample: [features]   → [outputs]
///   let outputs = kernel.eval_mut(&features);     // features × samples  → outputs × samples
/// ```
pub struct GraphKernel<'a, T, V> {
    nodes: &'a [GraphNode<T>],
    inner: KernelCache<V>,
}

impl<'a, T, V> GraphKernel<'a, T, V>
where
    V: Default + Clone,
{
    pub fn new<G>(graph: &'a G) -> Self
    where
        G: AsRef<[GraphNode<T>]>,
    {
        let cache = KernelCache::new(graph.as_ref());
        GraphKernel::from((graph, cache))
    }

    pub fn plan(&self) -> &GraphPlan {
        &self.inner.plan
    }

    pub fn take_cache(self) -> KernelCache<V> {
        self.inner
    }

    pub fn reset(&mut self) {
        self.inner.plan.compile(self.nodes);
        self.inner.row_values.clear();
        self.inner
            .row_values
            .resize(self.inner.plan.slots(), V::default());
    }
}

impl<'a, G, T, V> From<(&'a G, KernelCache<V>)> for GraphKernel<'a, T, V>
where
    G: AsRef<[GraphNode<T>]>,
    V: Default + Clone,
{
    fn from((graph, mut cache): (&'a G, KernelCache<V>)) -> Self {
        if !cache.is_ready {
            cache.plan.compile(graph.as_ref());
            cache.row_values.clear();
            cache.row_values.resize(cache.plan.slots(), V::default());
            cache.is_ready = true;
        }

        let nodes = graph.as_ref();
        GraphKernel {
            nodes,
            inner: cache,
        }
    }
}

impl<T, V> EvalIntoMut<Matrix<V>, Matrix<V>> for GraphKernel<'_, T, V>
where
    T: Eval<[V], V> + for<'c> EvalInto<[&'c [V]], [V]>,
    V: Copy + Default,
{
    /// Evaluates the graph over every sample: `features` is `features × samples` (one row per
    /// feature), and `outputs` is resized to `outputs × samples` (one row per output node).
    fn eval_into_mut(&mut self, features: &Matrix<V>, outputs: &mut Matrix<V>) {
        let n = features.cols();
        let KernelCache {
            plan,
            values,
            lagged,
            defaults,
            scratch,
            ..
        } = &mut self.inner;

        // One row per node (not just per evaluated slot), defaulted, so unreached trailing slots
        // read as default exactly like the row evaluator's untouched outputs.
        values.reshape_and_fill(plan.slots(), n, V::default());
        defaults.clear();
        defaults.resize(n, V::default());

        // A node with no incoming edges reads the sample's features: every feature row.
        let feature_columns: Vec<&[V]> = (0..features.rows()).map(|i| features.row(i)).collect();

        for step in plan.steps() {
            match step {
                Step::Column(slot) => eval_column(
                    self.nodes,
                    plan,
                    *slot as usize,
                    values,
                    lagged,
                    defaults,
                    &feature_columns,
                ),
                Step::Cycle(slots) => eval_cycle(self.nodes, plan, slots.clone(), values, scratch),
            }
        }

        outputs.reshape_and_fill(plan.output_slots().len(), n, V::default());
        for (j, &slot) in plan.output_slots().iter().enumerate() {
            outputs
                .row_mut(j)
                .copy_from_slice(values.row(slot as usize));
        }
    }
}

impl<T, V> EvalIntoMut<[V], [V]> for GraphKernel<'_, T, V>
where
    T: Eval<[V], V>,
    V: Copy + Default,
{
    /// Evaluates one sample. Runs the plan's [row order](GraphPlan::row_order), each node reading
    /// whatever its sources currently hold: this sample's value if the source already ran,
    /// otherwise the previous sample's (or default on the first call). `buffer` receives the
    /// output nodes' values, in output order.
    fn eval_into_mut(&mut self, input: &[V], buffer: &mut [V]) {
        let KernelCache {
            plan,
            row_values,
            scratch,
            ..
        } = &mut self.inner;

        for &slot in plan.row_order() {
            let slot = slot as usize;
            let node = &self.nodes[plan.node_of(slot)];
            let inputs = plan.inputs(slot);

            // No incoming edges means the node reads the sample's features directly.
            row_values[slot] = if inputs.is_empty() {
                node.eval(input)
            } else {
                scratch.clear();
                scratch.extend(inputs.iter().map(|i| row_values[i.slot as usize]));
                node.eval(scratch)
            };
        }

        for (out, &slot) in buffer.iter_mut().zip(plan.output_slots()) {
            *out = row_values[slot as usize];
        }
    }
}

impl<T, V> EvalMut<[V], Vec<V>> for GraphKernel<'_, T, V>
where
    T: Eval<[V], V>,
    V: Copy + Default,
{
    /// Evaluates one sample into a new `Vec` of the output nodes' values.
    fn eval_mut(&mut self, input: &[V]) -> Vec<V> {
        let mut outputs = vec![V::default(); self.inner.plan.output_slots().len()];
        self.eval_into_mut(input, &mut outputs);
        outputs
    }
}

impl<T, V> EvalMut<Matrix<V>, Matrix<V>> for GraphKernel<'_, T, V>
where
    T: Eval<[V], V> + for<'c> EvalInto<[&'c [V]], [V]>,
    V: Copy + Default,
{
    /// Evaluates the graph over every sample into a new `outputs × samples` matrix.
    fn eval_mut(&mut self, features: &Matrix<V>) -> Matrix<V> {
        let mut outputs = Matrix::default();
        self.eval_into_mut(features, &mut outputs);
        outputs
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
///           values[node][r] = node.eval(inputs)
/// ```
///
/// Inputs from outside the cycle are complete columns by the time this runs, so the only
/// sequential work is the cycle itself. A node on a cycle always has incoming edges, so it
/// never reads the features directly.
fn eval_cycle<T, V>(
    nodes: &[GraphNode<T>],
    plan: &GraphPlan,
    slots: Range<u32>,
    values: &mut Matrix<V>,
    scratch: &mut Vec<V>,
) where
    T: Eval<[V], V>,
    V: Copy + Default,
{
    for r in 0..values.cols() {
        for slot in slots.start as usize..slots.end as usize {
            let node = &nodes[plan.node_of(slot)];
            let inputs = plan.inputs(slot);
            debug_assert!(!inputs.is_empty(), "a node on a cycle has incoming edges");

            scratch.clear();
            scratch.extend(inputs.iter().map(|input| match input.at {
                When::Current => values[(input.slot as usize, r)],
                When::Previous if r > 0 => values[(input.slot as usize, r - 1)],
                When::Previous | When::Never => V::default(),
            }));

            values[(slot, r)] = node.eval(scratch);
        }
    }
}

impl<T, V> EvalInto<[Vec<V>], Vec<Vec<V>>> for Graph<T>
where
    T: Eval<[V], V>,
    V: Copy + Default,
{
    /// Evaluates the [Graph] with the given input `Vec<Vec<T>>`. Returns the output of the [Graph] as `Vec<Vec<T>>`.
    ///
    /// # Arguments
    /// * `input` - A `Vec<Vec<T>>` to evaluate the [Graph] with.
    ///
    /// # Returns
    /// * A `Vec<Vec<T>>` which is the output of the [Graph].
    #[inline]
    fn eval_into(&self, input: &[Vec<V>], buffer: &mut Vec<Vec<V>>) {
        let mut evaluator = GraphKernel::new(self);
        for i in 0..input.len() {
            evaluator.eval_into_mut(&input[i], &mut buffer[i]);
        }
    }
}

impl<T, V> Eval<[Vec<V>], Vec<Vec<V>>> for Graph<T>
where
    T: Eval<[V], V>,
    V: Copy + Default,
{
    /// Evaluates the [Graph] with the given input `Vec<Vec<T>>`. Returns the output of the [Graph] as `Vec<Vec<T>>`.
    /// This is intended to be used when evaluating a batch of inputs.
    ///
    /// # Arguments
    /// * `input` - A `Vec<Vec<T>>` to evaluate the [Graph] with.
    ///
    /// # Returns
    /// * A `Vec<Vec<T>>` which is the output of the [Graph].
    #[inline]
    fn eval(&self, input: &[Vec<V>]) -> Vec<Vec<V>> {
        let mut evaluator = GraphKernel::new(self);
        input
            .iter()
            .map(|input| evaluator.eval_mut(input))
            .collect()
    }
}

impl<T, V> Eval<[V], V> for GraphNode<T>
where
    T: Eval<[V], V>,
    V: Copy,
{
    /// Evaluates the [GraphNode] with the given input. Returns the output of the [GraphNode].
    /// # Arguments
    /// * `inputs` - A `Vec` of `V` to evaluate the [GraphNode] with.
    ///
    /// # Returns
    /// * A `V` which is the output of the [GraphNode].
    #[inline]
    fn eval(&self, inputs: &[V]) -> V {
        self.value().eval(inputs)
    }
}

impl<'a, T, V> EvalInto<[&'a [V]], [V]> for GraphNode<T>
where
    T: EvalInto<[&'a [V]], [V]>,
{
    /// Column form of [GraphNode]'s [Eval]: evaluates the node over every sample at once, where
    /// `inputs[i]` is the column of its i-th input and `buffer` receives one value per sample.
    #[inline]
    fn eval_into(&self, inputs: &[&'a [V]], buffer: &mut [V]) {
        self.value().eval_into(inputs, buffer)
    }
}

/// Computes one node's whole column (a node not on a cycle) via its column [EvalInto].
fn eval_column<T, V>(
    nodes: &[GraphNode<T>],
    plan: &GraphPlan,
    slot: usize,
    values: &mut Matrix<V>,
    lagged: &mut Vec<V>,
    defaults: &[V],
    feature_columns: &[&[V]],
) where
    T: for<'c> EvalInto<[&'c [V]], [V]>,
    V: Copy + Default,
{
    let n = values.cols();
    let node = &nodes[plan.node_of(slot)];
    let inputs = plan.inputs(slot);

    // Split `values` at this slot's row. Rust won't let us read and write `values` at the same
    // time, so `split_row_mut` hands back two non-overlapping views of the same memory, at no
    // cost: the finished rows as a read-only `MatrixView`, and this slot's row mutably.
    //
    //   computing slot 3 (add) in the module-docs example:
    //
    //   values.data():  [ slot 0 | slot 1 | slot 2 ][ slot 3 | slot 4 ]
    //                   └────────── prev ──────────┘└ out ─┘
    //                    MatrixView (slot rows × N)   &mut [V], this slot's N samples
    //
    // Slots are assigned in plan order, so every source this node reads is in `prev`.
    let (prev, out) = values.split_row_mut(slot);

    if inputs.is_empty() {
        node.eval_into(feature_columns, out);
        return;
    }

    // `Previous` inputs read their source one sample back: copy each source column lagged by
    // one, with default for sample 0.
    let n_lagged = inputs.iter().filter(|i| i.at == When::Previous).count();
    lagged.clear();
    lagged.resize(n_lagged * n, V::default());
    for (k, input) in inputs.iter().filter(|i| i.at == When::Previous).enumerate() {
        if n > 0 {
            lagged[k * n + 1..(k + 1) * n].copy_from_slice(&prev.row(input.slot as usize)[..n - 1]);
        }
    }

    // Gather the input columns: slices straight into `prev`, `lagged` or `defaults`, nothing
    // copied. Up to `STACK_INPUTS` live on the stack; wider nodes spill to a `Vec`.
    //
    //   inputs = [(1, Current), (2, Current)]  ──►  columns = [ prev.row(1), prev.row(2) ]
    //                                                           &[V; N]      &[V; N]
    let mut stack: [&[V]; STACK_INPUTS] = [&[]; STACK_INPUTS];
    let mut heap: Vec<&[V]> = Vec::new();
    let wide = inputs.len() > STACK_INPUTS;
    let mut k = 0;
    for (j, input) in inputs.iter().enumerate() {
        let column = match input.at {
            When::Current => prev.row(input.slot as usize),
            When::Previous => {
                k += 1;
                &lagged[(k - 1) * n..k * n]
            }
            When::Never => defaults,
        };
        if wide {
            heap.push(column);
        } else {
            stack[j] = column;
        }
    }
    let columns: &[&[V]] = if wide { &heap } else { &stack[..inputs.len()] };

    node.eval_into(columns, out);
}

// impl<'a, G, T, V> From<(&'a G, GraphEvalCache<V>)> for GraphEvaluator<'a, T, V>
// where
//     G: AsRef<[GraphNode<T>]>,
//     T: Eval<[V], V>,
//     V: Default + Clone,
// {
//     fn from((graph, cache): (&'a G, GraphEvalCache<V>)) -> Self {
//         if cache.eval_order.is_empty() || graph.as_ref().len() != cache.eval_order.len() {
//             return GraphEvaluator::new(graph);
//         }

//         GraphEvaluator {
//             nodes: graph.as_ref(),
//             inner: cache,
//         }
//     }
// }

// impl<'a, T, V> From<&'a Graph<T>> for GraphEvaluator<'a, T, V>
// where
//     T: Eval<[V], V>,
//     V: Default + Clone,
// {
//     fn from(graph: &'a Graph<T>) -> Self {
//         GraphEvaluator::new(graph)
//     }
// }

// /// A cache for storing intermediate results during graph evaluation.
// ///
// /// This cache is used to store the inputs and outputs of each node in the graph
// /// during evaluation, allowing for more efficient re-evaluation of nodes when
// /// their inputs change. If we want to save a graph's evaluation between different evals,
// /// we need to keep track of the inputs and outputs from previous runs incase of recurrent
// /// structures. This cache is the answer to that.
// #[derive(Clone, Debug, PartialEq)]
// #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
// pub struct GraphEvalCache<V> {
//     eval_order: Vec<usize>,
//     outputs: Vec<V>,
//     inputs: Vec<V>,
//     input_ranges: Vec<(usize, usize)>,
//     output_indices: Vec<usize>,
// }

// /// [GraphEvaluator] is a struct that is used to evaluate a [Graph] of [GraphNode]'s. It uses the [GraphIterator]
// /// to traverse the [Graph] in a pseudo-topological order and evaluate the nodes in the correct order.
// pub struct GraphEvaluator<'a, T, V> {
//     nodes: &'a [GraphNode<T>],
//     inner: GraphEvalCache<V>,
// }

// impl<'a, T, V> GraphEvaluator<'a, T, V>
// where
//     T: Eval<[V], V>,
//     V: Default + Clone,
// {
//     /// Creates a new [GraphEvaluator] with the given [Graph]. We pre-allocate the necessary
//     /// storage for inputs and outputs based on the structure of the graph on creation.
//     /// This way, we can reuse the same evaluator for multiple evaluations of the same graph
//     /// without needing to reallocate memory each time.
//     ///
//     /// # Arguments
//     /// * graph - The [Graph] to reduce.
//     #[inline]
//     pub fn new<N>(graph: &'a N) -> Self
//     where
//         N: AsRef<[GraphNode<T>]>,
//     {
//         let nodes = graph.as_ref();

//         let mut total_inputs = 0;
//         let mut input_ranges = Vec::with_capacity(nodes.len());

//         for node in nodes {
//             let k = node.incoming().len();
//             input_ranges.push((total_inputs, total_inputs + k));
//             total_inputs += k;
//         }

//         let output_indices = graph
//             .get_nodes_of_type(NodeType::Output)
//             .map(|n| n.index())
//             .collect::<Vec<usize>>();

//         GraphEvaluator {
//             nodes,
//             inner: GraphEvalCache {
//                 inputs: vec![V::default(); total_inputs],
//                 outputs: vec![V::default(); nodes.len()],
//                 eval_order: nodes.iter_topological().map(|n| n.index()).collect(),
//                 input_ranges,
//                 output_indices,
//             },
//         }
//     }

//     pub fn take_cache(self) -> GraphEvalCache<V> {
//         self.inner
//     }
// }

// impl<T, V> EvalMut<[V], Vec<V>> for GraphEvaluator<'_, T, V>
// where
//     T: Eval<[V], V>,
//     V: Copy + Default,
// {
//     #[inline]
//     fn eval_mut(&mut self, input: &[V]) -> Vec<V> {
//         let out_len = self.inner.output_indices.len();
//         let mut buffer = vec![V::default(); out_len];
//         self.eval_into_mut(input, &mut buffer[..]);
//         buffer
//     }
// }

// impl<T, V> EvalIntoMut<[V], [V]> for GraphEvaluator<'_, T, V>
// where
//     T: Eval<[V], V>,
//     V: Copy + Default,
// {
//     #[inline]
//     fn eval_into_mut(&mut self, input: &[V], buffer: &mut [V]) {
//         for &index in self.inner.eval_order.iter() {
//             let node = &self.nodes[index];
//             let incoming = node.incoming();

//             if incoming.is_empty() {
//                 self.inner.outputs[index] = node.eval(input);
//             } else {
//                 let (start, end) = &self.inner.input_ranges[index];
//                 let buf = &mut self.inner.inputs[*start..*end];

//                 for (dst, &src_idx) in buf.iter_mut().zip(incoming.iter()) {
//                     *dst = self.inner.outputs[src_idx];
//                 }

//                 self.inner.outputs[index] = node.eval(buf);
//             }
//         }

//         for (count, &idx) in self.inner.output_indices.iter().enumerate() {
//             buffer[count] = self.inner.outputs[idx];
//         }
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::{Graph, Op};

//     fn round(value: f32, places: u32) -> f32 {
//         let factor = 10_f32.powi(places as i32);
//         (value * factor).round() / factor
//     }

//     #[test]
//     fn test_graph_eval_simple() {
//         let mut graph = Graph::<Op<f32>>::default();

//         let idx_one = graph.insert(NodeType::Input, Op::var(0));
//         let idx_two = graph.insert(NodeType::Input, Op::constant(5_f32));
//         let idx_three = graph.insert(NodeType::Vertex, Op::add());
//         let idx_four = graph.insert(NodeType::Output, Op::linear());

//         graph
//             .attach(idx_one, idx_three)
//             .attach(idx_two, idx_three)
//             .attach(idx_three, idx_four);

//         let six = graph.eval(&[vec![1_f32]]);
//         let seven = graph.eval(&[vec![2_f32]]);
//         let eight = graph.eval(&[vec![3_f32]]);

//         assert_eq!(six, vec![vec![6_f32]]);
//         assert_eq!(seven, vec![vec![7_f32]]);
//         assert_eq!(eight, vec![vec![8_f32]]);
//         assert_eq!(graph.len(), 4);
//     }

//     #[test]
//     fn test_graph_eval_recurrent() {
//         let mut graph = Graph::<Op<f32>>::default();

//         graph.insert(NodeType::Input, Op::var(0));
//         graph.insert(NodeType::Vertex, Op::diff());
//         graph.insert(NodeType::Output, Op::sigmoid());
//         graph.insert(NodeType::Edge, Op::weight_with(-1.41));
//         graph.insert(NodeType::Vertex, Op::sigmoid());
//         graph.insert(NodeType::Vertex, Op::exp());
//         graph.insert(NodeType::Edge, Op::weight_with(-1.10));
//         graph.insert(NodeType::Vertex, Op::exp());
//         graph.insert(NodeType::Vertex, Op::exp());
//         graph.insert(NodeType::Vertex, Op::div());

//         graph.attach(0, 1);
//         graph.attach(1, 1);
//         graph.attach(4, 1);
//         graph.attach(7, 1);
//         graph.attach(8, 1);
//         graph.attach(1, 2);
//         graph.attach(3, 2);
//         graph.attach(6, 2);
//         graph.attach(5, 3);
//         graph.attach(1, 4);
//         graph.attach(0, 5);
//         graph.attach(9, 6);
//         graph.attach(4, 7);
//         graph.attach(7, 8);
//         graph.attach(0, 9);
//         graph.attach(9, 9);

//         graph.set_cycles(vec![]);

//         let mut evaluator = GraphEvaluator::new(&graph);

//         let out1 = evaluator.eval_mut(&vec![0.0])[0];
//         let out2 = evaluator.eval_mut(&vec![0.0])[0];
//         let out3 = evaluator.eval_mut(&vec![0.0])[0];
//         let out4 = evaluator.eval_mut(&vec![1.0])[0];
//         let out5 = evaluator.eval_mut(&vec![0.0])[0];
//         let out6 = evaluator.eval_mut(&vec![0.0])[0];
//         let out7 = evaluator.eval_mut(&vec![0.0])[0];

//         assert_eq!(round(out1, 3), 0.196);
//         assert_eq!(round(out2, 3), 0.000);
//         assert_eq!(round(out3, 3), 0.902);
//         assert_eq!(round(out4, 3), 0.000);
//         assert_eq!(round(out5, 3), 1.000);
//         assert_eq!(round(out6, 3), 0.000);
//         assert_eq!(round(out7, 3), 1.000);
//     }
// }

// // use super::{Graph, GraphNode, iter::GraphIterator};
// // use crate::{
// //     Eval, EvalMut, NodeType,
// //     eval::{EvalInto, EvalIntoMut},
// //     node::Node,
// // };
// // use radiate_core::Valid;
// // #[cfg(feature = "serde")]
// // use serde::{Deserialize, Serialize};

// // /// One step of a compiled graph program.
// // ///
// // /// Instructions are stored in evaluation order, so instruction `i` writes slot `i` of the
// // /// output buffer. `src_start..src_end` is the range in [GraphEvalCache::srcs] holding the
// // /// slots this instruction reads from. An empty range means the node reads the graph input
// // /// directly (input / constant nodes).
// // #[derive(Clone, Copy, Debug, PartialEq, Eq)]
// // #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
// // struct Instr {
// //     node: u32,
// //     src_start: u32,
// //     src_end: u32,
// // }

// // /// A compiled, reusable evaluation plan for a graph plus the state needed to evaluate it.
// // ///
// // /// The graph's nodes are compiled once into a flat instruction tape in pseudo-topological
// // /// order. Each node's incoming indices are remapped to "slots" (positions in that order), so
// // /// evaluation walks contiguous arrays and never touches a [GraphNode].
// // ///
// // /// `outputs` holds the last value produced by every slot. That is also what makes recurrent
// // /// graphs work: a back edge reads its source's value from the previous evaluation. Keeping
// // /// this cache between evaluations (see [GraphEvaluator::take_cache]) preserves that state.
// // #[derive(Clone, Debug, PartialEq)]
// // #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
// // pub struct GraphEvalCache<V> {
// //     instrs: Vec<Instr>,
// //     srcs: Vec<u32>,
// //     outputs: Vec<V>,
// //     scratch: Vec<V>,
// //     output_slots: Vec<u32>,
// //     /// Structural fingerprint of the graph this cache was compiled for. Computed lazily in
// //     /// [GraphEvaluator::take_cache], since only reused caches need it. `None` means unknown,
// //     /// so the cache is never reused.
// //     fingerprint: Option<u64>,
// // }

// // impl<V> GraphEvalCache<V> {
// //     /// Compiles `nodes` into an instruction tape with fresh (default) state.
// //     fn compile<T>(nodes: &[GraphNode<T>]) -> Self
// //     where
// //         V: Default + Clone,
// //     {
// //         debug_assert!(nodes.len() <= u32::MAX as usize);

// //         let order: Vec<usize> = nodes.iter_topological().map(|n| n.index()).collect();

// //         // Nodes the iterator yields get slots 0..order.len() in evaluation order. Any node it
// //         // doesn't yield (e.g. every node of an invalid graph) gets a trailing slot that is never
// //         // written, so reading it returns V::default(), the same as the original evaluator.
// //         let mut slot_of = vec![u32::MAX; nodes.len()];
// //         for (slot, &index) in order.iter().enumerate() {
// //             slot_of[index] = slot as u32;
// //         }
// //         let mut next_slot = order.len() as u32;
// //         for slot in slot_of.iter_mut() {
// //             if *slot == u32::MAX {
// //                 *slot = next_slot;
// //                 next_slot += 1;
// //             }
// //         }

// //         let mut instrs = Vec::with_capacity(order.len());
// //         let mut srcs = Vec::new();
// //         let mut max_fan_in = 0;

// //         for &index in &order {
// //             let incoming = nodes[index].incoming();
// //             let start = srcs.len() as u32;
// //             srcs.extend(incoming.iter().map(|&src| slot_of[src]));
// //             max_fan_in = max_fan_in.max(incoming.len());

// //             instrs.push(Instr {
// //                 node: index as u32,
// //                 src_start: start,
// //                 src_end: srcs.len() as u32,
// //             });
// //         }

// //         let output_slots = nodes
// //             .get_nodes_of_type(NodeType::Output)
// //             .map(|n| slot_of[n.index()])
// //             .collect();

// //         GraphEvalCache {
// //             instrs,
// //             srcs,
// //             outputs: vec![V::default(); nodes.len()],
// //             scratch: vec![V::default(); max_fan_in],
// //             output_slots,
// //             fingerprint: None,
// //         }
// //     }

// //     /// True when this cache was compiled for a graph with exactly this structure.
// //     #[inline]
// //     fn matches<T>(&self, nodes: &[GraphNode<T>]) -> bool {
// //         self.outputs.len() == nodes.len() && self.fingerprint == Some(structure_fingerprint(nodes))
// //     }
// // }

// // /// FNV-1a over everything that decides the compiled tape: node count, each node's incoming
// // /// edges, and the recurrence/validity flags the topological iterator branches on. Ops are
// // /// deliberately left out. They're borrowed from the graph at evaluation time, so swapping an
// // /// op doesn't invalidate the tape.
// // fn structure_fingerprint<T>(nodes: &[GraphNode<T>]) -> u64 {
// //     #[inline(always)]
// //     fn mix(hash: u64, value: u64) -> u64 {
// //         (hash ^ value).wrapping_mul(0x0000_0100_0000_01b3)
// //     }

// //     let mut hash = mix(0xcbf2_9ce4_8422_2325, nodes.len() as u64);
// //     for node in nodes {
// //         let incoming = node.incoming();
// //         let flags = (incoming.len() as u64)
// //             | ((node.is_recurrent() as u64) << 32)
// //             | ((node.is_valid() as u64) << 33);

// //         hash = mix(hash, flags);
// //         for &src in incoming {
// //             hash = mix(hash, src as u64);
// //         }
// //     }

// //     hash
// // }

// // /// [GraphEvaluator] evaluates a [Graph] of [GraphNode]s. On creation it compiles the graph
// // /// (in the pseudo-topological order given by [GraphIterator]) into a flat instruction tape,
// // /// then evaluates by walking that tape.
// // pub struct GraphEvaluator<'a, T, V> {
// //     nodes: &'a [GraphNode<T>],
// //     /// Each node's op, in evaluation order (parallel to `inner.instrs`).
// //     ops: Vec<&'a T>,
// //     inner: GraphEvalCache<V>,
// // }

// // impl<'a, T, V> GraphEvaluator<'a, T, V>
// // where
// //     T: Eval<[V], V>,
// //     V: Default + Clone,
// // {
// //     /// Creates a new [GraphEvaluator] for the given graph, compiling it into an instruction
// //     /// tape and pre-allocating all evaluation storage. Reuse the evaluator to evaluate the
// //     /// same graph many times without recompiling or reallocating.
// //     ///
// //     /// # Arguments
// //     /// * graph - The [Graph] (or slice of [GraphNode]s) to evaluate.
// //     #[inline]
// //     pub fn new<N>(graph: &'a N) -> Self
// //     where
// //         N: AsRef<[GraphNode<T>]>,
// //     {
// //         let nodes = graph.as_ref();
// //         Self::with_cache(nodes, GraphEvalCache::compile(nodes))
// //     }

// //     #[inline]
// //     fn with_cache(nodes: &'a [GraphNode<T>], inner: GraphEvalCache<V>) -> Self {
// //         let ops = inner
// //             .instrs
// //             .iter()
// //             .map(|ins| nodes[ins.node as usize].value())
// //             .collect();

// //         GraphEvaluator { nodes, ops, inner }
// //     }

// //     /// Consumes the evaluator and returns its compiled tape and state, so it can be handed
// //     /// back later via `GraphEvaluator::from((&graph, cache))`. Recurrent state carries over,
// //     /// and recompiling is skipped if the graph's structure hasn't changed.
// //     pub fn take_cache(self) -> GraphEvalCache<V> {
// //         let mut inner = self.inner;
// //         if inner.fingerprint.is_none() {
// //             inner.fingerprint = Some(structure_fingerprint(self.nodes));
// //         }
// //         inner
// //     }
// // }

// // impl<T, V> EvalMut<[V], Vec<V>> for GraphEvaluator<'_, T, V>
// // where
// //     T: Eval<[V], V>,
// //     V: Copy + Default,
// // {
// //     #[inline]
// //     fn eval_mut(&mut self, input: &[V]) -> Vec<V> {
// //         let mut buffer = vec![V::default(); self.inner.output_slots.len()];
// //         self.eval_into_mut(input, &mut buffer[..]);
// //         buffer
// //     }
// // }

// // impl<T, V> EvalIntoMut<[V], [V]> for GraphEvaluator<'_, T, V>
// // where
// //     T: Eval<[V], V>,
// //     V: Copy + Default,
// // {
// //     #[inline]
// //     fn eval_into_mut(&mut self, input: &[V], buffer: &mut [V]) {
// //         let GraphEvalCache {
// //             instrs,
// //             srcs,
// //             outputs,
// //             scratch,
// //             output_slots,
// //             ..
// //         } = &mut self.inner;

// //         for (slot, (ins, op)) in instrs.iter().zip(self.ops.iter()).enumerate() {
// //             let (start, end) = (ins.src_start as usize, ins.src_end as usize);

// //             let value = if start == end {
// //                 op.eval(input)
// //             } else {
// //                 let buf = &mut scratch[..end - start];
// //                 for (dst, &src) in buf.iter_mut().zip(&srcs[start..end]) {
// //                     *dst = outputs[src as usize];
// //                 }
// //                 op.eval(buf)
// //             };

// //             outputs[slot] = value;
// //         }

// //         for (dst, &slot) in buffer.iter_mut().zip(output_slots.iter()) {
// //             *dst = outputs[slot as usize];
// //         }
// //     }
// // }

// // impl<T, V> EvalInto<[Vec<V>], Vec<Vec<V>>> for Graph<T>
// // where
// //     T: Eval<[V], V>,
// //     V: Copy + Default,
// // {
// //     /// Evaluates the [Graph] for each row of `input`, writing row `i`'s outputs into
// //     /// `buffer[i]`. The graph is compiled once and state carries from row to row.
// //     #[inline]
// //     fn eval_into(&self, input: &[Vec<V>], buffer: &mut Vec<Vec<V>>) {
// //         let mut evaluator = GraphEvaluator::new(self);
// //         for (row, out) in input.iter().zip(buffer.iter_mut()) {
// //             evaluator.eval_into_mut(row, out);
// //         }
// //     }
// // }

// // impl<T, V> Eval<[Vec<V>], Vec<Vec<V>>> for Graph<T>
// // where
// //     T: Eval<[V], V>,
// //     V: Copy + Default,
// // {
// //     /// Evaluates the [Graph] for each row of `input` and returns one output `Vec` per row.
// //     /// The graph is compiled once and state carries from row to row.
// //     #[inline]
// //     fn eval(&self, input: &[Vec<V>]) -> Vec<Vec<V>> {
// //         let mut evaluator = GraphEvaluator::new(self);
// //         input.iter().map(|row| evaluator.eval_mut(row)).collect()
// //     }
// // }

// // impl<T, V> Eval<[V], V> for GraphNode<T>
// // where
// //     T: Eval<[V], V>,
// //     V: Copy,
// // {
// //     /// Evaluates the [GraphNode]'s value with the given inputs.
// //     #[inline]
// //     fn eval(&self, inputs: &[V]) -> V {
// //         self.value().eval(inputs)
// //     }
// // }

// // impl<'a, G, T, V> From<(&'a G, GraphEvalCache<V>)> for GraphEvaluator<'a, T, V>
// // where
// //     G: AsRef<[GraphNode<T>]>,
// //     T: Eval<[V], V>,
// //     V: Default + Clone,
// // {
// //     /// Reuses `cache` (tape and recurrent state) if it was compiled for a graph with the same
// //     /// structure, otherwise compiles fresh. The old check only compared node counts, so a
// //     /// rewired graph with the same length silently reused a stale evaluation order.
// //     fn from((graph, cache): (&'a G, GraphEvalCache<V>)) -> Self {
// //         let nodes = graph.as_ref();
// //         if cache.matches(nodes) {
// //             GraphEvaluator::with_cache(nodes, cache)
// //         } else {
// //             GraphEvaluator::new(graph)
// //         }
// //     }
// // }

// // impl<'a, T, V> From<&'a Graph<T>> for GraphEvaluator<'a, T, V>
// // where
// //     T: Eval<[V], V>,
// //     V: Default + Clone,
// // {
// //     fn from(graph: &'a Graph<T>) -> Self {
// //         GraphEvaluator::new(graph)
// //     }
// // }

// // #[cfg(test)]
// // mod tests {
// //     use super::*;
// //     use crate::{Graph, Op};

// //     fn round(value: f32, places: u32) -> f32 {
// //         let factor = 10_f32.powi(places as i32);
// //         (value * factor).round() / factor
// //     }

// //     fn simple_graph() -> Graph<Op<f32>> {
// //         let mut graph = Graph::<Op<f32>>::default();

// //         let idx_one = graph.insert(NodeType::Input, Op::var(0));
// //         let idx_two = graph.insert(NodeType::Input, Op::constant(5_f32));
// //         let idx_three = graph.insert(NodeType::Vertex, Op::add());
// //         let idx_four = graph.insert(NodeType::Output, Op::linear());

// //         graph
// //             .attach(idx_one, idx_three)
// //             .attach(idx_two, idx_three)
// //             .attach(idx_three, idx_four);

// //         graph
// //     }

// //     fn recurrent_graph() -> Graph<Op<f32>> {
// //         let mut graph = Graph::<Op<f32>>::default();

// //         graph.insert(NodeType::Input, Op::var(0));
// //         graph.insert(NodeType::Vertex, Op::diff());
// //         graph.insert(NodeType::Output, Op::sigmoid());
// //         graph.insert(NodeType::Edge, Op::weight_with(-1.41));
// //         graph.insert(NodeType::Vertex, Op::sigmoid());
// //         graph.insert(NodeType::Vertex, Op::exp());
// //         graph.insert(NodeType::Edge, Op::weight_with(-1.10));
// //         graph.insert(NodeType::Vertex, Op::exp());
// //         graph.insert(NodeType::Vertex, Op::exp());
// //         graph.insert(NodeType::Vertex, Op::div());

// //         graph.attach(0, 1);
// //         graph.attach(1, 1);
// //         graph.attach(4, 1);
// //         graph.attach(7, 1);
// //         graph.attach(8, 1);
// //         graph.attach(1, 2);
// //         graph.attach(3, 2);
// //         graph.attach(6, 2);
// //         graph.attach(5, 3);
// //         graph.attach(1, 4);
// //         graph.attach(0, 5);
// //         graph.attach(9, 6);
// //         graph.attach(4, 7);
// //         graph.attach(7, 8);
// //         graph.attach(0, 9);
// //         graph.attach(9, 9);

// //         graph.set_cycles(vec![]);
// //         graph
// //     }

// //     const RECURRENT_INPUTS: [f32; 7] = [0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0];
// //     const RECURRENT_EXPECTED: [f32; 7] = [0.196, 0.000, 0.902, 0.000, 1.000, 0.000, 1.000];

// //     #[test]
// //     fn test_graph_eval_simple() {
// //         let graph = simple_graph();

// //         let six = graph.eval(&[vec![1_f32]]);
// //         let seven = graph.eval(&[vec![2_f32]]);
// //         let eight = graph.eval(&[vec![3_f32]]);

// //         assert_eq!(six, vec![vec![6_f32]]);
// //         assert_eq!(seven, vec![vec![7_f32]]);
// //         assert_eq!(eight, vec![vec![8_f32]]);
// //         assert_eq!(graph.len(), 4);
// //     }

// //     #[test]
// //     fn test_graph_eval_recurrent() {
// //         let graph = recurrent_graph();
// //         let mut evaluator = GraphEvaluator::new(&graph);

// //         for (input, expected) in RECURRENT_INPUTS.iter().zip(RECURRENT_EXPECTED) {
// //             let out = evaluator.eval_mut(&vec![*input])[0];
// //             assert_eq!(round(out, 3), expected);
// //         }
// //     }

// //     #[test]
// //     fn test_cache_round_trip_keeps_recurrent_state() {
// //         let graph = recurrent_graph();
// //         let mut cache = None;

// //         // Rebuild the evaluator from the cache on every step, as StatefulGraph does. The
// //         // sequence must match a single long-lived evaluator exactly.
// //         for (input, expected) in RECURRENT_INPUTS.iter().zip(RECURRENT_EXPECTED) {
// //             let mut evaluator = match cache.take() {
// //                 Some(c) => GraphEvaluator::from((&graph, c)),
// //                 None => GraphEvaluator::new(&graph),
// //             };

// //             let out = evaluator.eval_mut(&vec![*input])[0];
// //             assert_eq!(round(out, 3), expected);
// //             cache = Some(evaluator.take_cache());
// //         }
// //     }

// //     #[test]
// //     fn test_stale_cache_is_recompiled_for_rewired_graph() {
// //         let graph = simple_graph();
// //         let mut evaluator = GraphEvaluator::new(&graph);
// //         assert_eq!(evaluator.eval_mut(&[1_f32]), vec![6_f32]);
// //         let cache = evaluator.take_cache();

// //         // Same node count, different wiring: the output now also reads the constant.
// //         let mut rewired = Graph::<Op<f32>>::default();
// //         rewired.insert(NodeType::Input, Op::var(0));
// //         rewired.insert(NodeType::Input, Op::constant(5_f32));
// //         rewired.insert(NodeType::Vertex, Op::add());
// //         rewired.insert(NodeType::Output, Op::add());
// //         rewired.attach(0, 2).attach(1, 2).attach(2, 3).attach(1, 3);

// //         assert_eq!(rewired.len(), graph.len());

// //         let mut evaluator = GraphEvaluator::from((&rewired, cache));
// //         assert_eq!(evaluator.eval_mut(&[1_f32]), vec![11_f32]);
// //     }

// //     #[test]
// //     fn test_invalid_graph_evaluates_to_default() {
// //         // An input with no outgoing edges and an output with no incoming edges: the
// //         // topological iterator yields nothing, so every output stays at V::default().
// //         let mut graph = Graph::<Op<f32>>::default();
// //         graph.insert(NodeType::Input, Op::var(0));
// //         graph.insert(NodeType::Output, Op::linear());

// //         let mut evaluator = GraphEvaluator::new(&graph);
// //         assert_eq!(evaluator.eval_mut(&[3_f32]), vec![0_f32]);
// //     }
// // }
