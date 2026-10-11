// use super::{Graph, GraphNode, iter::GraphIterator};
// use crate::{
//     Eval, EvalMut, NodeType,
//     eval::{EvalInto, EvalIntoMut},
//     node::Node,
// };
// #[cfg(feature = "serde")]
// use serde::{Deserialize, Serialize};

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

// impl<T, V> EvalInto<[Vec<V>], Vec<Vec<V>>> for Graph<T>
// where
//     T: Eval<[V], V>,
//     V: Copy + Default,
// {
//     /// Evaluates the [Graph] with the given input `Vec<Vec<T>>`. Returns the output of the [Graph] as `Vec<Vec<T>>`.
//     ///
//     /// # Arguments
//     /// * `input` - A `Vec<Vec<T>>` to evaluate the [Graph] with.
//     ///
//     /// # Returns
//     /// * A `Vec<Vec<T>>` which is the output of the [Graph].
//     #[inline]
//     fn eval_into(&self, input: &[Vec<V>], buffer: &mut Vec<Vec<V>>) {
//         let mut evaluator = GraphEvaluator::new(self);
//         for i in 0..input.len() {
//             evaluator.eval_into_mut(&input[i], &mut buffer[i]);
//         }
//     }
// }

// impl<T, V> Eval<[Vec<V>], Vec<Vec<V>>> for Graph<T>
// where
//     T: Eval<[V], V>,
//     V: Copy + Default,
// {
//     /// Evaluates the [Graph] with the given input `Vec<Vec<T>>`. Returns the output of the [Graph] as `Vec<Vec<T>>`.
//     /// This is intended to be used when evaluating a batch of inputs.
//     ///
//     /// # Arguments
//     /// * `input` - A `Vec<Vec<T>>` to evaluate the [Graph] with.
//     ///
//     /// # Returns
//     /// * A `Vec<Vec<T>>` which is the output of the [Graph].
//     #[inline]
//     fn eval(&self, input: &[Vec<V>]) -> Vec<Vec<V>> {
//         let mut evaluator = GraphEvaluator::new(self);
//         input
//             .iter()
//             .map(|input| evaluator.eval_mut(input))
//             .collect()
//     }
// }

// impl<T, V> Eval<[V], V> for GraphNode<T>
// where
//     T: Eval<[V], V>,
//     V: Copy,
// {
//     /// Evaluates the [GraphNode] with the given input. Returns the output of the [GraphNode].
//     /// # Arguments
//     /// * `inputs` - A `Vec` of `V` to evaluate the [GraphNode] with.
//     ///
//     /// # Returns
//     /// * A `V` which is the output of the [GraphNode].
//     #[inline]
//     fn eval(&self, inputs: &[V]) -> V {
//         self.value().eval(inputs)
//     }
// }

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

use super::{Graph, GraphNode, iter::GraphIterator};
use crate::{
    Eval, EvalMut, NodeType,
    eval::{EvalInto, EvalIntoMut},
    node::Node,
};
use radiate_core::Valid;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// One step of a compiled graph program.
///
/// Instructions are stored in evaluation order, so instruction `i` writes slot `i` of the
/// output buffer. `src_start..src_end` is the range in [GraphEvalCache::srcs] holding the
/// slots this instruction reads from. An empty range means the node reads the graph input
/// directly (input / constant nodes).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
struct Instr {
    node: u32,
    src_start: u32,
    src_end: u32,
}

/// A compiled, reusable evaluation plan for a graph plus the state needed to evaluate it.
///
/// The graph's nodes are compiled once into a flat instruction tape in pseudo-topological
/// order. Each node's incoming indices are remapped to "slots" (positions in that order), so
/// evaluation walks contiguous arrays and never touches a [GraphNode].
///
/// `outputs` holds the last value produced by every slot. That is also what makes recurrent
/// graphs work: a back edge reads its source's value from the previous evaluation. Keeping
/// this cache between evaluations (see [GraphEvaluator::take_cache]) preserves that state.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GraphEvalCache<V> {
    instrs: Vec<Instr>,
    srcs: Vec<u32>,
    outputs: Vec<V>,
    scratch: Vec<V>,
    output_slots: Vec<u32>,
    /// Structural fingerprint of the graph this cache was compiled for. Computed lazily in
    /// [GraphEvaluator::take_cache], since only reused caches need it. `None` means unknown,
    /// so the cache is never reused.
    fingerprint: Option<u64>,
}

impl<V> GraphEvalCache<V> {
    /// Compiles `nodes` into an instruction tape with fresh (default) state.
    fn compile<T>(nodes: &[GraphNode<T>]) -> Self
    where
        V: Default + Clone,
    {
        debug_assert!(nodes.len() <= u32::MAX as usize);

        let order: Vec<usize> = nodes.iter_topological().map(|n| n.index()).collect();

        // Nodes the iterator yields get slots 0..order.len() in evaluation order. Any node it
        // doesn't yield (e.g. every node of an invalid graph) gets a trailing slot that is never
        // written, so reading it returns V::default(), the same as the original evaluator.
        let mut slot_of = vec![u32::MAX; nodes.len()];
        for (slot, &index) in order.iter().enumerate() {
            slot_of[index] = slot as u32;
        }
        let mut next_slot = order.len() as u32;
        for slot in slot_of.iter_mut() {
            if *slot == u32::MAX {
                *slot = next_slot;
                next_slot += 1;
            }
        }

        let mut instrs = Vec::with_capacity(order.len());
        let mut srcs = Vec::new();
        let mut max_fan_in = 0;

        for &index in &order {
            let incoming = nodes[index].incoming();
            let start = srcs.len() as u32;
            srcs.extend(incoming.iter().map(|&src| slot_of[src]));
            max_fan_in = max_fan_in.max(incoming.len());

            instrs.push(Instr {
                node: index as u32,
                src_start: start,
                src_end: srcs.len() as u32,
            });
        }

        let output_slots = nodes
            .get_nodes_of_type(NodeType::Output)
            .map(|n| slot_of[n.index()])
            .collect();

        GraphEvalCache {
            instrs,
            srcs,
            outputs: vec![V::default(); nodes.len()],
            scratch: vec![V::default(); max_fan_in],
            output_slots,
            fingerprint: None,
        }
    }

    /// True when this cache was compiled for a graph with exactly this structure.
    #[inline]
    fn matches<T>(&self, nodes: &[GraphNode<T>]) -> bool {
        self.outputs.len() == nodes.len() && self.fingerprint == Some(structure_fingerprint(nodes))
    }
}

/// FNV-1a over everything that decides the compiled tape: node count, each node's incoming
/// edges, and the recurrence/validity flags the topological iterator branches on. Ops are
/// deliberately left out. They're borrowed from the graph at evaluation time, so swapping an
/// op doesn't invalidate the tape.
fn structure_fingerprint<T>(nodes: &[GraphNode<T>]) -> u64 {
    #[inline(always)]
    fn mix(hash: u64, value: u64) -> u64 {
        (hash ^ value).wrapping_mul(0x0000_0100_0000_01b3)
    }

    let mut hash = mix(0xcbf2_9ce4_8422_2325, nodes.len() as u64);
    for node in nodes {
        let incoming = node.incoming();
        let flags = (incoming.len() as u64)
            | ((node.is_recurrent() as u64) << 32)
            | ((node.is_valid() as u64) << 33);

        hash = mix(hash, flags);
        for &src in incoming {
            hash = mix(hash, src as u64);
        }
    }

    hash
}

/// [GraphEvaluator] evaluates a [Graph] of [GraphNode]s. On creation it compiles the graph
/// (in the pseudo-topological order given by [GraphIterator]) into a flat instruction tape,
/// then evaluates by walking that tape.
pub struct GraphEvaluator<'a, T, V> {
    nodes: &'a [GraphNode<T>],
    /// Each node's op, in evaluation order (parallel to `inner.instrs`).
    ops: Vec<&'a T>,
    inner: GraphEvalCache<V>,
}

impl<'a, T, V> GraphEvaluator<'a, T, V>
where
    T: Eval<[V], V>,
    V: Default + Clone,
{
    /// Creates a new [GraphEvaluator] for the given graph, compiling it into an instruction
    /// tape and pre-allocating all evaluation storage. Reuse the evaluator to evaluate the
    /// same graph many times without recompiling or reallocating.
    ///
    /// # Arguments
    /// * graph - The [Graph] (or slice of [GraphNode]s) to evaluate.
    #[inline]
    pub fn new<N>(graph: &'a N) -> Self
    where
        N: AsRef<[GraphNode<T>]>,
    {
        let nodes = graph.as_ref();
        Self::with_cache(nodes, GraphEvalCache::compile(nodes))
    }

    #[inline]
    fn with_cache(nodes: &'a [GraphNode<T>], inner: GraphEvalCache<V>) -> Self {
        let ops = inner
            .instrs
            .iter()
            .map(|ins| nodes[ins.node as usize].value())
            .collect();

        GraphEvaluator { nodes, ops, inner }
    }

    /// Consumes the evaluator and returns its compiled tape and state, so it can be handed
    /// back later via `GraphEvaluator::from((&graph, cache))`. Recurrent state carries over,
    /// and recompiling is skipped if the graph's structure hasn't changed.
    pub fn take_cache(self) -> GraphEvalCache<V> {
        let mut inner = self.inner;
        if inner.fingerprint.is_none() {
            inner.fingerprint = Some(structure_fingerprint(self.nodes));
        }
        inner
    }
}

impl<T, V> EvalMut<[V], Vec<V>> for GraphEvaluator<'_, T, V>
where
    T: Eval<[V], V>,
    V: Copy + Default,
{
    #[inline]
    fn eval_mut(&mut self, input: &[V]) -> Vec<V> {
        let mut buffer = vec![V::default(); self.inner.output_slots.len()];
        self.eval_into_mut(input, &mut buffer[..]);
        buffer
    }
}

impl<T, V> EvalIntoMut<[V], [V]> for GraphEvaluator<'_, T, V>
where
    T: Eval<[V], V>,
    V: Copy + Default,
{
    #[inline]
    fn eval_into_mut(&mut self, input: &[V], buffer: &mut [V]) {
        let GraphEvalCache {
            instrs,
            srcs,
            outputs,
            scratch,
            output_slots,
            ..
        } = &mut self.inner;

        for (slot, (ins, op)) in instrs.iter().zip(self.ops.iter()).enumerate() {
            let (start, end) = (ins.src_start as usize, ins.src_end as usize);

            let value = if start == end {
                op.eval(input)
            } else {
                let buf = &mut scratch[..end - start];
                for (dst, &src) in buf.iter_mut().zip(&srcs[start..end]) {
                    *dst = outputs[src as usize];
                }
                op.eval(buf)
            };

            outputs[slot] = value;
        }

        for (dst, &slot) in buffer.iter_mut().zip(output_slots.iter()) {
            *dst = outputs[slot as usize];
        }
    }
}

impl<T, V> EvalInto<[Vec<V>], Vec<Vec<V>>> for Graph<T>
where
    T: Eval<[V], V>,
    V: Copy + Default,
{
    /// Evaluates the [Graph] for each row of `input`, writing row `i`'s outputs into
    /// `buffer[i]`. The graph is compiled once and state carries from row to row.
    #[inline]
    fn eval_into(&self, input: &[Vec<V>], buffer: &mut Vec<Vec<V>>) {
        let mut evaluator = GraphEvaluator::new(self);
        for (row, out) in input.iter().zip(buffer.iter_mut()) {
            evaluator.eval_into_mut(row, out);
        }
    }
}

impl<T, V> Eval<[Vec<V>], Vec<Vec<V>>> for Graph<T>
where
    T: Eval<[V], V>,
    V: Copy + Default,
{
    /// Evaluates the [Graph] for each row of `input` and returns one output `Vec` per row.
    /// The graph is compiled once and state carries from row to row.
    #[inline]
    fn eval(&self, input: &[Vec<V>]) -> Vec<Vec<V>> {
        let mut evaluator = GraphEvaluator::new(self);
        input.iter().map(|row| evaluator.eval_mut(row)).collect()
    }
}

impl<T, V> Eval<[V], V> for GraphNode<T>
where
    T: Eval<[V], V>,
    V: Copy,
{
    /// Evaluates the [GraphNode]'s value with the given inputs.
    #[inline]
    fn eval(&self, inputs: &[V]) -> V {
        self.value().eval(inputs)
    }
}

impl<'a, G, T, V> From<(&'a G, GraphEvalCache<V>)> for GraphEvaluator<'a, T, V>
where
    G: AsRef<[GraphNode<T>]>,
    T: Eval<[V], V>,
    V: Default + Clone,
{
    /// Reuses `cache` (tape and recurrent state) if it was compiled for a graph with the same
    /// structure, otherwise compiles fresh. The old check only compared node counts, so a
    /// rewired graph with the same length silently reused a stale evaluation order.
    fn from((graph, cache): (&'a G, GraphEvalCache<V>)) -> Self {
        let nodes = graph.as_ref();
        if cache.matches(nodes) {
            GraphEvaluator::with_cache(nodes, cache)
        } else {
            GraphEvaluator::new(graph)
        }
    }
}

impl<'a, T, V> From<&'a Graph<T>> for GraphEvaluator<'a, T, V>
where
    T: Eval<[V], V>,
    V: Default + Clone,
{
    fn from(graph: &'a Graph<T>) -> Self {
        GraphEvaluator::new(graph)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Graph, Op};

    fn round(value: f32, places: u32) -> f32 {
        let factor = 10_f32.powi(places as i32);
        (value * factor).round() / factor
    }

    fn simple_graph() -> Graph<Op<f32>> {
        let mut graph = Graph::<Op<f32>>::default();

        let idx_one = graph.insert(NodeType::Input, Op::var(0));
        let idx_two = graph.insert(NodeType::Input, Op::constant(5_f32));
        let idx_three = graph.insert(NodeType::Vertex, Op::add());
        let idx_four = graph.insert(NodeType::Output, Op::linear());

        graph
            .attach(idx_one, idx_three)
            .attach(idx_two, idx_three)
            .attach(idx_three, idx_four);

        graph
    }

    fn recurrent_graph() -> Graph<Op<f32>> {
        let mut graph = Graph::<Op<f32>>::default();

        graph.insert(NodeType::Input, Op::var(0));
        graph.insert(NodeType::Vertex, Op::diff());
        graph.insert(NodeType::Output, Op::sigmoid());
        graph.insert(NodeType::Edge, Op::weight_with(-1.41));
        graph.insert(NodeType::Vertex, Op::sigmoid());
        graph.insert(NodeType::Vertex, Op::exp());
        graph.insert(NodeType::Edge, Op::weight_with(-1.10));
        graph.insert(NodeType::Vertex, Op::exp());
        graph.insert(NodeType::Vertex, Op::exp());
        graph.insert(NodeType::Vertex, Op::div());

        graph.attach(0, 1);
        graph.attach(1, 1);
        graph.attach(4, 1);
        graph.attach(7, 1);
        graph.attach(8, 1);
        graph.attach(1, 2);
        graph.attach(3, 2);
        graph.attach(6, 2);
        graph.attach(5, 3);
        graph.attach(1, 4);
        graph.attach(0, 5);
        graph.attach(9, 6);
        graph.attach(4, 7);
        graph.attach(7, 8);
        graph.attach(0, 9);
        graph.attach(9, 9);

        graph.set_cycles(vec![]);
        graph
    }

    const RECURRENT_INPUTS: [f32; 7] = [0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0];
    const RECURRENT_EXPECTED: [f32; 7] = [0.196, 0.000, 0.902, 0.000, 1.000, 0.000, 1.000];

    #[test]
    fn test_graph_eval_simple() {
        let graph = simple_graph();

        let six = graph.eval(&[vec![1_f32]]);
        let seven = graph.eval(&[vec![2_f32]]);
        let eight = graph.eval(&[vec![3_f32]]);

        assert_eq!(six, vec![vec![6_f32]]);
        assert_eq!(seven, vec![vec![7_f32]]);
        assert_eq!(eight, vec![vec![8_f32]]);
        assert_eq!(graph.len(), 4);
    }

    #[test]
    fn test_graph_eval_recurrent() {
        let graph = recurrent_graph();
        let mut evaluator = GraphEvaluator::new(&graph);

        for (input, expected) in RECURRENT_INPUTS.iter().zip(RECURRENT_EXPECTED) {
            let out = evaluator.eval_mut(&vec![*input])[0];
            assert_eq!(round(out, 3), expected);
        }
    }

    #[test]
    fn test_cache_round_trip_keeps_recurrent_state() {
        let graph = recurrent_graph();
        let mut cache = None;

        // Rebuild the evaluator from the cache on every step, as StatefulGraph does. The
        // sequence must match a single long-lived evaluator exactly.
        for (input, expected) in RECURRENT_INPUTS.iter().zip(RECURRENT_EXPECTED) {
            let mut evaluator = match cache.take() {
                Some(c) => GraphEvaluator::from((&graph, c)),
                None => GraphEvaluator::new(&graph),
            };

            let out = evaluator.eval_mut(&vec![*input])[0];
            assert_eq!(round(out, 3), expected);
            cache = Some(evaluator.take_cache());
        }
    }

    #[test]
    fn test_stale_cache_is_recompiled_for_rewired_graph() {
        let graph = simple_graph();
        let mut evaluator = GraphEvaluator::new(&graph);
        assert_eq!(evaluator.eval_mut(&[1_f32]), vec![6_f32]);
        let cache = evaluator.take_cache();

        // Same node count, different wiring: the output now also reads the constant.
        let mut rewired = Graph::<Op<f32>>::default();
        rewired.insert(NodeType::Input, Op::var(0));
        rewired.insert(NodeType::Input, Op::constant(5_f32));
        rewired.insert(NodeType::Vertex, Op::add());
        rewired.insert(NodeType::Output, Op::add());
        rewired.attach(0, 2).attach(1, 2).attach(2, 3).attach(1, 3);

        assert_eq!(rewired.len(), graph.len());

        let mut evaluator = GraphEvaluator::from((&rewired, cache));
        assert_eq!(evaluator.eval_mut(&[1_f32]), vec![11_f32]);
    }

    #[test]
    fn test_invalid_graph_evaluates_to_default() {
        // An input with no outgoing edges and an output with no incoming edges: the
        // topological iterator yields nothing, so every output stays at V::default().
        let mut graph = Graph::<Op<f32>>::default();
        graph.insert(NodeType::Input, Op::var(0));
        graph.insert(NodeType::Output, Op::linear());

        let mut evaluator = GraphEvaluator::new(&graph);
        assert_eq!(evaluator.eval_mut(&[3_f32]), vec![0_f32]);
    }
}
