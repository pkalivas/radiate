use super::{
    GraphNode,
    compile::{GraphPlan, Step, When},
};
use crate::{Eval, EvalInto, EvalIntoMut, EvalMut, Graph};
use radiate_utils::Matrix;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

//  // Rows → columns: feature i becomes contiguous row i.
//     let mut columns = std::mem::take(&mut self.columns);
//     columns.reshape_and_fill(width, samples, V::default());
//     for (r, sample) in features.iter().enumerate() {
//         for (i, &x) in sample.iter().enumerate() {
//             columns[(i, r)] = x;
//         }
//     }

//     let mut predictions = std::mem::take(&mut self.predictions);
//     self.eval_columns(nodes, &columns, &mut predictions);

//     // Columns → rows.
//     for (r, out) in outputs.iter_mut().enumerate() {
//         for (j, o) in out.iter_mut().enumerate() {
//             *o = predictions[(j, r)];
//         }
//     }

//     self.columns = columns;
//     self.predictions = predictions;
// }

/// Input columns are collected into a stack array up to this fan-in, so the common case doesn't
/// allocate per node; wider nodes spill to the heap.
const STACK_INPUTS: usize = 8;

/// A [GraphEvaluator]'s reusable state: the compiled plan, each node's latest value, and the
/// buffers it runs in.
///
/// The cache can evaluate its graph directly ([GraphEvalCache::eval_row],
/// [GraphEvalCache::eval_batch]), so whatever owns a graph can own its cache too, compile once,
/// and evaluate without holding a [GraphEvaluator] borrow (see `StatefulGraph`).
///
/// - Keep one between calls on the *same* graph ([GraphEvaluator::take_cache],
///   `GraphEvaluator::from((&graph, cache))`) to carry recurrent state forward.
/// - Pass one to *another* graph ([GraphEvaluator::with_cache]) to reuse its allocations.
#[derive(Default, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GraphEvalCache<V> {
    plan: GraphPlan,
    outputs: Vec<V>,
    inputs: Vec<V>,
    values: Matrix<V>,
    lagged: Vec<V>,
    defaults: Vec<V>,
}

impl<V> GraphEvalCache<V>
where
    V: Default + Clone,
{
    pub fn new<T>(nodes: &[GraphNode<T>]) -> Self {
        let mut cache = GraphEvalCache::default();
        cache.compile(nodes);
        cache
    }

    pub fn plan(&self) -> &GraphPlan {
        &self.plan
    }

    fn compile<T>(&mut self, nodes: &[GraphNode<T>]) {
        self.plan.compile(nodes);
        self.outputs.clear();
        self.outputs.resize(nodes.len(), V::default());
        self.inputs.clear();
        self.inputs.resize(self.plan.edges(), V::default());
    }
}
impl<V> GraphEvalCache<V>
where
    V: Default + Clone,
{
    /// Clears recurrent state, keeping the compiled plan and buffers.
    pub fn reset_state(&mut self) {
        self.outputs.fill(V::default());
    }
}

/// [GraphEvaluator] evaluates a [Graph] of [GraphNode]s through [EvalIntoMut] / [EvalMut], two
/// ways:
///
/// - **One sample at a time**, `[V]` in, `[V]` out: runs the nodes in the [GraphIterator]'s
///   pseudo-topological order, keeping each node's last value between calls, so recurrent graphs
///   carry state from one sample to the next.
/// - **A whole batch at once**, `features × samples` in, `outputs × samples` out: runs the plan's
///   column schedule, a column at a time wherever the graph allows it.
pub struct GraphEvaluator<'a, T, V> {
    nodes: &'a [GraphNode<T>],
    inner: GraphEvalCache<V>,
}

impl<'a, T, V> GraphEvaluator<'a, T, V>
where
    V: Default + Clone,
{
    /// Compiles `graph` into a new [GraphEvaluator]. Only the cheap row plan is built here; the
    /// column schedule waits for the first batch.
    pub fn new<G>(graph: &'a G) -> Self
    where
        G: AsRef<[GraphNode<T>]>,
    {
        let nodes = graph.as_ref();
        GraphEvaluator {
            nodes,
            inner: GraphEvalCache::new(nodes),
        }
    }

    /// Compiles `graph` into `cache`, reusing its allocations, with every node's state at
    /// default. `cache` can come from any graph.
    pub fn with_cache<G>(graph: &'a G, mut cache: GraphEvalCache<V>) -> Self
    where
        G: AsRef<[GraphNode<T>]>,
    {
        let nodes = graph.as_ref();
        cache.compile(nodes);
        GraphEvaluator {
            nodes,
            inner: cache,
        }
    }

    pub fn plan(&self) -> &GraphPlan {
        &self.inner.plan
    }

    pub fn take_cache(self) -> GraphEvalCache<V> {
        self.inner
    }

    /// Resets every node's state to default, as if no sample had been evaluated yet.
    pub fn reset(&mut self) {
        self.inner.reset_state();
    }
}

impl<T, V> EvalMut<[V], Vec<V>> for GraphEvaluator<'_, T, V>
where
    T: Eval<[V], V>,
    V: Copy + Default,
{
    #[inline]
    fn eval_mut(&mut self, input: &[V]) -> Vec<V> {
        let mut buffer = vec![V::default(); self.inner.plan.output_indices().len()];
        self.eval_into_mut(input, &mut buffer);
        buffer
    }
}

impl<T, V> EvalIntoMut<[V], [V]> for GraphEvaluator<'_, T, V>
where
    T: Eval<[V], V>,
    V: Copy + Default,
{
    /// Evaluates one sample; see [GraphEvalCache::eval_row].
    #[inline]
    fn eval_into_mut(&mut self, input: &[V], buffer: &mut [V]) {
        process_single_row_into(&mut self.inner, self.nodes, input, buffer);
    }
}

impl<T, V> EvalMut<Matrix<V>, Matrix<V>> for GraphEvaluator<'_, T, V>
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

impl<T, V> EvalIntoMut<Matrix<V>, Matrix<V>> for GraphEvaluator<'_, T, V>
where
    T: Eval<[V], V> + for<'c> EvalInto<[&'c [V]], [V]>,
    V: Copy + Default,
{
    /// Evaluates every sample at once; see [GraphEvalCache::eval_batch].
    fn eval_into_mut(&mut self, features: &Matrix<V>, outputs: &mut Matrix<V>) {
        process_batch_column_major(&mut self.inner, self.nodes, features, outputs);
    }
}

impl<'a, G, T, V> From<(&'a G, GraphEvalCache<V>)> for GraphEvaluator<'a, T, V>
where
    G: AsRef<[GraphNode<T>]>,
    V: Default + Clone,
{
    /// Creates a new [GraphEvaluator] from a graph and a cache, reusing the cache's allocations if possible.
    /// If the cache's plan does not match the graph, a new cache is created.
    /// This is aimed at reusing existing caches when evaluating multiple graphs with similar structures.
    fn from((graph, mut cache): (&'a G, GraphEvalCache<V>)) -> Self {
        if cache.plan.order().len() != graph.as_ref().len() {
            cache = GraphEvalCache::new(graph.as_ref());
        }

        let nodes = graph.as_ref();
        GraphEvaluator {
            nodes,
            inner: cache,
        }
    }
}

impl<'a, T, V> From<&'a Graph<T>> for GraphEvaluator<'a, T, V>
where
    V: Default + Clone,
{
    fn from(graph: &'a Graph<T>) -> Self {
        GraphEvaluator::new(graph)
    }
}

/// Evaluates one sample of the graph this cache was compiled from. Runs the plan's
/// [order](GraphPlan::order), each node reading whatever its sources currently hold: this
/// sample's value if the source already ran, otherwise the previous sample's (or default on
/// the first call). `buffer` receives the output nodes' values, in output order.
///
/// `nodes` must be the graph the cache was compiled from.
#[inline]
pub fn process_single_row_into<T, V>(
    cache: &mut GraphEvalCache<V>,
    nodes: &[GraphNode<T>],
    input: &[V],
    buffer: &mut [V],
) where
    T: Eval<[V], V>,
    V: Copy + Default,
{
    let GraphEvalCache {
        plan,
        outputs,
        inputs,
        ..
    } = cache;

    for &index in plan.order() {
        let node = &nodes[index];
        let incoming = node.incoming();

        // No incoming edges means the node reads the sample's features directly.
        outputs[index] = if incoming.is_empty() {
            node.eval(input)
        } else {
            let buf = &mut inputs[plan.inputs(index)];
            for (dst, &src) in buf.iter_mut().zip(incoming) {
                *dst = outputs[src];
            }
            node.eval(buf)
        };
    }

    for (out, &index) in buffer.iter_mut().zip(plan.output_indices()) {
        *out = outputs[index];
    }
}

pub fn process_batch_column_major<T, V>(
    cache: &mut GraphEvalCache<V>,
    nodes: &[GraphNode<T>],
    features: &Matrix<V>,
    outputs: &mut Matrix<V>,
) where
    T: Eval<[V], V> + for<'c> EvalInto<[&'c [V]], [V]>,
    V: Copy + Default,
{
    let cols = features.cols();
    let GraphEvalCache {
        plan,
        inputs,
        values,
        lagged,
        defaults,
        ..
    } = cache;

    plan.schedule(nodes);
    let plan = &*plan;

    // One row per node, defaulted, so nodes that never run read as default exactly like
    // their untouched state one sample at a time.
    values.reshape_and_fill(nodes.len(), cols, V::default());
    defaults.clear();
    defaults.resize(cols, V::default());

    // A node with no incoming edges reads the sample's features - ie: giving input
    // nodes their inputs to vectorize over the samples.
    // Features is transposed, so each row is the original dataset's feature vector for that feature.
    // Hence why we can collect a SmallVec of slices
    let feature_columns = (0..features.rows())
        .map(|i| features.row(i))
        .collect::<SmallVec<[&[V]; STACK_INPUTS]>>();

    for step in plan.steps() {
        match step {
            Step::Column(index) => process_column_major(
                *index,
                nodes,
                plan,
                values,
                lagged,
                defaults,
                &feature_columns,
            ),
            Step::Cycle(range) => {
                let cycle = plan.cycle_nodes(range.clone());
                process_cycle_column_major(cycle, nodes, plan, values, inputs)
            }
        }
    }

    outputs.reshape_and_fill(plan.output_indices().len(), cols, V::default());
    for (j, &index) in plan.output_indices().iter().enumerate() {
        outputs.row_mut(j).copy_from_slice(values.row(index));
    }
}

/// Steps a cycle's nodes through the samples together, exactly like one-sample-at-a-time
/// evaluation but restricted to the cycle:
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
/// of the edge-indexed `inputs` buffer, the same buffer one-sample-at-a-time evaluation uses.
fn process_cycle_column_major<T, V>(
    cycle: &[usize],
    nodes: &[GraphNode<T>],
    plan: &GraphPlan,
    values: &mut Matrix<V>,
    inputs: &mut [V],
) where
    T: Eval<[V], V>,
    V: Copy + Default,
{
    for r in 0..values.cols() {
        for &index in cycle {
            let node = &nodes[index];
            let buf = &mut inputs[plan.inputs(index)];
            for ((dst, &src), when) in buf.iter_mut().zip(node.incoming()).zip(plan.when(index)) {
                *dst = match when {
                    When::Current => values[(src, r)],
                    When::Previous if r > 0 => values[(src, r - 1)],
                    When::Previous | When::Never => V::default(),
                };
            }
            values[(index, r)] = node.eval(buf);
        }
    }
}
/// Computes one node's whole column (a node not on a cycle) via its column [EvalInto].
fn process_column_major<T, V>(
    index: usize,
    nodes: &[GraphNode<T>],
    plan: &GraphPlan,
    values: &mut Matrix<V>,
    lagged: &mut Vec<V>,
    defaults: &[V],
    feature_columns: &[&[V]],
) where
    T: for<'c> EvalInto<[&'c [V]], [V]>,
    V: Copy + Default,
{
    let n = values.cols();
    let node = &nodes[index];
    let incoming = node.incoming();

    let (above, out, below) = values.split_row_mut(index);
    let row = |src: usize| {
        if src < index {
            above.row(src)
        } else {
            below.row(src - index - 1)
        }
    };

    if incoming.is_empty() {
        node.eval_into(feature_columns, out);
        return;
    }

    let when = plan.when(index);
    let previous = || {
        incoming
            .iter()
            .zip(when)
            .filter(|(_, w)| **w == When::Previous)
            .map(|(&src, _)| src)
    };
    lagged.clear();
    lagged.resize(previous().count() * n, V::default());
    if n > 0 {
        for (k, src) in previous().enumerate() {
            lagged[k * n + 1..(k + 1) * n].copy_from_slice(&row(src)[..n - 1]);
        }
    }

    // Gather the input columns: slices straight into `values`, `lagged` or `defaults`
    // Up to `STACK_INPUTS` live on the stack
    let mut lagged_columns = lagged.chunks_exact(n.max(1));
    let columns = incoming
        .iter()
        .zip(when)
        .map(|(&src, w)| match w {
            When::Current => row(src),
            When::Previous => lagged_columns.next().unwrap_or(&[]),
            When::Never => defaults,
        })
        .collect::<SmallVec<[&[V]; STACK_INPUTS]>>();

    node.eval_into(&columns, out);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{NodeType, Op};

    fn assert_rows_match_batch(graph: &Graph<Op<f32>>, samples: &[Vec<f32>]) -> Vec<Vec<f32>> {
        let rows = graph.eval(samples);

        let width = samples.first().map_or(0, |s| s.len());
        let mut features = Matrix::default();
        features.reshape_and_fill(width, samples.len(), 0.0);
        for (r, sample) in samples.iter().enumerate() {
            for (f, &x) in sample.iter().enumerate() {
                features[(f, r)] = x;
            }
        }

        let batch = GraphEvaluator::new(graph).eval_mut(&features);
        assert_eq!(batch.rows(), rows.first().map_or(0, |r| r.len()));
        for (r, row) in rows.iter().enumerate() {
            for (j, &value) in row.iter().enumerate() {
                assert_eq!(
                    value.to_bits(),
                    batch[(j, r)].to_bits(),
                    "sample {r}, output {j}: row {value} vs batch {}",
                    batch[(j, r)]
                );
            }
        }

        rows
    }

    fn store(inputs: usize) -> Vec<(NodeType, Vec<Op<f32>>)> {
        vec![
            (NodeType::Input, (0..inputs).map(Op::var).collect()),
            (NodeType::Edge, vec![Op::weight_with(0.7), Op::identity()]),
            (
                NodeType::Vertex,
                vec![Op::sigmoid(), Op::tanh(), Op::mul(), Op::add()],
            ),
            (NodeType::Output, vec![Op::sigmoid()]),
        ]
    }

    fn samples(width: usize, n: usize) -> Vec<Vec<f32>> {
        (0..n)
            .map(|r| {
                (0..width)
                    .map(|f| ((r * 7 + f * 3) % 11) as f32 * 0.37 - 1.5)
                    .collect()
            })
            .collect()
    }

    #[test]
    fn test_graph_eval_simple() {
        let mut graph = Graph::<Op<f32>>::default();

        let idx_one = graph.insert(NodeType::Input, Op::var(0));
        let idx_two = graph.insert(NodeType::Input, Op::constant(5_f32));
        let idx_three = graph.insert(NodeType::Vertex, Op::add());
        let idx_four = graph.insert(NodeType::Output, Op::linear());

        graph
            .attach(idx_one, idx_three)
            .attach(idx_two, idx_three)
            .attach(idx_three, idx_four);

        let outputs = assert_rows_match_batch(&graph, &[vec![1.0], vec![2.0], vec![3.0]]);
        assert_eq!(outputs, vec![vec![6.0], vec![7.0], vec![8.0]]);
    }

    #[test]
    fn test_self_loop_carries_state() {
        // out = x + previous out: a running sum.
        let mut graph = Graph::<Op<f32>>::default();

        let x = graph.insert(NodeType::Input, Op::var(0));
        let acc = graph.insert(NodeType::Vertex, Op::sum());
        let out = graph.insert(NodeType::Output, Op::linear());

        graph.attach(x, acc).attach(acc, acc).attach(acc, out);

        let outputs = assert_rows_match_batch(&graph, &[vec![1.0], vec![2.0], vec![3.0]]);
        assert_eq!(outputs, vec![vec![1.0], vec![3.0], vec![6.0]]);
    }

    #[test]
    fn test_back_edge_cycle() {
        // x0 → add → h → sigmoid, with h → add as a back edge.
        let mut graph = Graph::<Op<f32>>::default();

        let x0 = graph.insert(NodeType::Input, Op::var(0));
        let x1 = graph.insert(NodeType::Input, Op::var(1));
        let add = graph.insert(NodeType::Vertex, Op::sum());
        let h = graph.insert(NodeType::Vertex, Op::tanh());
        let out = graph.insert(NodeType::Output, Op::sigmoid());

        graph
            .attach(x0, add)
            .attach(x1, add)
            .attach(h, add)
            .attach(add, h)
            .attach(h, out);
        graph.set_cycles(vec![]);

        assert_rows_match_batch(&graph, &samples(2, 50));

        let mut evaluator = GraphEvaluator::new(&graph);
        evaluator.eval_mut(&Matrix::new(vec![0.0_f32, 0.0]).reshape(2, 1));
        assert!(evaluator.plan().is_cyclic());
    }

    #[test]
    fn test_builder_graphs() {
        let graphs = [
            Graph::directed(3, 2, store(3)),
            Graph::recurrent(3, 2, store(3)),
            Graph::weighted_directed(3, 2, store(3)),
            Graph::weighted_recurrent(3, 2, store(3)),
            Graph::lstm(3, 2, store(3)),
            Graph::gru(3, 2, store(3)),
        ];

        for graph in &graphs {
            assert_rows_match_batch(graph, &samples(3, 40));
        }
    }

    #[test]
    fn test_cache_round_trip_keeps_state() {
        let mut graph = Graph::<Op<f32>>::default();

        let x = graph.insert(NodeType::Input, Op::var(0));
        let acc = graph.insert(NodeType::Vertex, Op::sum());
        let out = graph.insert(NodeType::Output, Op::linear());

        graph.attach(x, acc).attach(acc, acc).attach(acc, out);

        let mut evaluator = GraphEvaluator::new(&graph);
        assert_eq!(evaluator.eval_mut(&[1.0][..]), vec![1.0]);
        let cache = evaluator.take_cache();

        // Resuming keeps the running sum; a batch in between doesn't disturb it.
        let mut evaluator = GraphEvaluator::from((&graph, cache));
        evaluator.eval_mut(&Matrix::new(vec![5.0_f32, 5.0]).reshape(1, 2));
        assert_eq!(evaluator.eval_mut(&[2.0][..]), vec![3.0]);

        // with_cache starts over.
        let mut evaluator = GraphEvaluator::with_cache(&graph, evaluator.take_cache());
        assert_eq!(evaluator.eval_mut(&[2.0][..]), vec![2.0]);

        evaluator.reset();
        assert_eq!(evaluator.eval_mut(&[4.0][..]), vec![4.0]);
    }

    #[test]
    fn test_with_cache_from_another_graph() {
        let small = Graph::directed(2, 1, store(2));
        let large = Graph::lstm(3, 2, store(3));

        let features = Matrix::new(vec![0.5_f32, -1.0, 2.0]).reshape(3, 1);
        let mut evaluator = GraphEvaluator::new(&large);
        let want = evaluator.eval_mut(&features);

        let mut evaluator = GraphEvaluator::new(&small);
        evaluator.eval_mut(&Matrix::new(vec![1.0_f32, 2.0]).reshape(2, 1));
        let mut evaluator = GraphEvaluator::with_cache(&large, evaluator.take_cache());
        assert_eq!(evaluator.eval_mut(&features), want);
    }

    #[test]
    fn test_invalid_graph_evaluates_to_default() {
        let mut graph = Graph::<Op<f32>>::default();
        let a = graph.insert(NodeType::Vertex, Op::sum());
        let b = graph.insert(NodeType::Output, Op::sigmoid());
        graph.attach(a, b).attach(b, a);

        assert_rows_match_batch(&graph, &samples(1, 5));
    }
}
