//! PROTOTYPE: column-wise evaluation of acyclic graphs.
//!
//! The row evaluator ([GraphEvaluator](super::GraphEvaluator)) runs every node once per data
//! row. Here each node runs over *every* row at once, writing into a slot-major buffer
//! (`cols[slot * rows + r]`), so dispatch happens once per node instead of once per node per row.
//!
//! This is only equivalent to row-by-row evaluation when every node reads slots computed before
//! it in the same row. Back edges, self loops and unreached nodes depend on state carried
//! between rows, so [eval_columns] returns `None` for those graphs and the caller falls back to
//! the row evaluator. See `graph.md` for the investigation and numbers.

use super::{GraphNode, iter::GraphIterator};
use crate::{
    Eval, NodeType, Op,
    node::Node,
    ops::{OpFloat, op_names},
};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ColMode {
    /// Column layout, but every node calls `op.eval` once per row.
    Generic,
    /// Column layout with per-op kernels for the common ops; others fall back to [ColMode::Generic].
    Kernels,
}

#[derive(Default, Debug)]
pub struct ColStats {
    pub graphs: usize,
    pub columnable: usize,
    pub kernel_nodes: usize,
    pub fallback_nodes: usize,
}

/// Reusable buffers for [eval_columns]. Keep one around across graphs to avoid reallocating.
#[derive(Default)]
pub struct ColBuf<F> {
    cols: Vec<F>,
    scratch: Vec<F>,
    order: Vec<usize>,
    slot_of: Vec<u32>,
    srcs: Vec<u32>,
    ranges: Vec<(u32, u32)>,
    output_slots: Vec<u32>,
}

impl<F: Copy> ColBuf<F> {
    /// Output slots of the last graph evaluated, in output-node order.
    #[inline]
    pub fn output_slots(&self) -> &[u32] {
        &self.output_slots
    }

    /// Value of `slot` for row `r`, from the last graph evaluated over `n_rows` rows.
    #[inline]
    pub fn value(&self, slot: u32, n_rows: usize, r: usize) -> F {
        self.cols[slot as usize * n_rows + r]
    }
}

/// Evaluates `nodes` over all `rows`. Returns `false` when the graph isn't safely columnable
/// (see the module docs); the buffer contents are then meaningless and the caller should use
/// the row evaluator. On success, read results with [ColBuf::output_slots] and [ColBuf::value].
pub fn eval_columns<F: OpFloat>(
    nodes: &[GraphNode<Op<F>>],
    rows: &[&[F]],
    buf: &mut ColBuf<F>,
    mode: ColMode,
    stats: &mut ColStats,
) -> bool {
    stats.graphs += 1;
    if !compile(nodes, buf) {
        return false;
    }
    stats.columnable += 1;

    let n_rows = rows.len();
    buf.cols.clear();
    buf.cols.resize(nodes.len() * n_rows, F::default());

    for (slot, &index) in buf.order.iter().enumerate() {
        let op = nodes[index].value();
        let (start, end) = buf.ranges[slot];
        let srcs = &buf.srcs[start as usize..end as usize];

        let (prev, rest) = buf.cols.split_at_mut(slot * n_rows);
        let out = &mut rest[..n_rows];

        if mode == ColMode::Kernels && kernel(op, srcs, prev, rows, out) {
            stats.kernel_nodes += 1;
            continue;
        }
        stats.fallback_nodes += 1;

        if srcs.is_empty() {
            for (o, row) in out.iter_mut().zip(rows) {
                *o = op.eval(row);
            }
        } else {
            if buf.scratch.len() < srcs.len() {
                buf.scratch.resize(srcs.len(), F::default());
            }
            let scratch = &mut buf.scratch[..srcs.len()];
            for (r, o) in out.iter_mut().enumerate() {
                for (d, &s) in scratch.iter_mut().zip(srcs) {
                    *d = prev[s as usize * n_rows + r];
                }
                *o = op.eval(scratch);
            }
        }
    }

    true
}

/// Lays the graph out in the same pseudo-topological order the row evaluator uses, remapping
/// incoming edges to evaluation-order slots. Returns `false` if any node reads a slot that
/// isn't computed before it.
fn compile<F>(nodes: &[GraphNode<Op<F>>], buf: &mut ColBuf<F>) -> bool {
    buf.order.clear();
    buf.order
        .extend(nodes.iter_topological().map(|n| n.index()));

    // Nodes the iterator never yields get trailing slots that are never written, so they
    // read as default, like the row evaluator. Reading one makes the graph non-columnable.
    buf.slot_of.clear();
    buf.slot_of.resize(nodes.len(), u32::MAX);
    for (slot, &index) in buf.order.iter().enumerate() {
        buf.slot_of[index] = slot as u32;
    }
    let unreached = buf.slot_of.iter_mut().filter(|s| **s == u32::MAX);
    for (next, slot) in (buf.order.len() as u32..).zip(unreached) {
        *slot = next;
    }

    buf.srcs.clear();
    buf.ranges.clear();
    for (slot, &index) in buf.order.iter().enumerate() {
        let start = buf.srcs.len() as u32;
        for &src in nodes[index].incoming() {
            let s = buf.slot_of[src];
            if s as usize >= slot {
                return false;
            }
            buf.srcs.push(s);
        }
        buf.ranges.push((start, buf.srcs.len() as u32));
    }

    buf.output_slots.clear();
    buf.output_slots.extend(
        nodes
            .get_nodes_of_type(NodeType::Output)
            .map(|n| buf.slot_of[n.index()]),
    );

    true
}

#[inline(always)]
fn clamp<F: OpFloat>(value: F) -> F {
    if value.is_nan() {
        return F::ZERO;
    }
    value.clamp(-F::MAX_VALUE, F::MAX_VALUE)
}

#[inline(always)]
fn col<F>(prev: &[F], s: u32, n: usize) -> &[F] {
    &prev[s as usize * n..s as usize * n + n]
}

/// Sum of the source columns into `out`, matching `aggregate` in `ops/math.rs` bit-for-bit:
/// 1..=5 inputs sum left to right from the first input, more than 5 fold from zero.
#[inline(always)]
fn aggregate_into<F: OpFloat>(srcs: &[u32], prev: &[F], out: &mut [F]) {
    let n = out.len();
    if srcs.is_empty() {
        out.fill(F::ZERO);
        return;
    }

    if srcs.len() <= 5 {
        out.copy_from_slice(col(prev, srcs[0], n));
    } else {
        out.fill(F::ZERO);
        for (o, &x) in out.iter_mut().zip(col(prev, srcs[0], n)) {
            *o = *o + x;
        }
    }

    for &s in &srcs[1..] {
        for (o, &x) in out.iter_mut().zip(col(prev, s, n)) {
            *o = *o + x;
        }
    }
}

#[inline(always)]
fn binary<F: OpFloat>(srcs: &[u32], prev: &[F], out: &mut [F], f: impl Fn(F, F) -> F) -> bool {
    if srcs.len() < 2 {
        return false;
    }
    let n = out.len();
    let (a, b) = (col(prev, srcs[0], n), col(prev, srcs[1], n));
    for ((o, &x), &y) in out.iter_mut().zip(a).zip(b) {
        *o = f(x, y);
    }
    true
}

#[inline(always)]
fn activation<F: OpFloat>(srcs: &[u32], prev: &[F], out: &mut [F], f: impl Fn(F) -> F) -> bool {
    aggregate_into(srcs, prev, out);
    for o in out.iter_mut() {
        *o = f(*o);
    }
    true
}

/// Column kernel for `op`, or `false` if there isn't one (caller falls back to per-row eval).
/// Each kernel repeats the scalar op's float operations in the same order, so results are
/// bit-identical to the row evaluator.
///
/// PROTOTYPE: ops are matched by name, so a user-defined op reusing a built-in name would get
/// the wrong kernel. A real version should dispatch on something the op owns.
fn kernel<F: OpFloat>(op: &Op<F>, srcs: &[u32], prev: &[F], rows: &[&[F]], out: &mut [F]) -> bool {
    let n = out.len();
    match op {
        Op::Var(_, index, _) if srcs.is_empty() => {
            for (o, row) in out.iter_mut().zip(rows) {
                *o = row[*index];
            }
            true
        }
        Op::Const(_, v) => {
            out.fill(*v);
            true
        }
        Op::Value(name, _, param, _) if *name == op_names::WEIGHT && !srcs.is_empty() => {
            let w = *param.data();
            for (o, &x) in out.iter_mut().zip(col(prev, srcs[0], n)) {
                *o = clamp(x * w);
            }
            true
        }
        Op::Fn(name, _, _) => match *name {
            op_names::IDENTITY if !srcs.is_empty() => {
                out.copy_from_slice(col(prev, srcs[0], n));
                true
            }
            op_names::ADD => binary(srcs, prev, out, |a, b| clamp(a + b)),
            op_names::SUB => binary(srcs, prev, out, |a, b| clamp(a - b)),
            op_names::MUL => binary(srcs, prev, out, |a, b| clamp(a * b)),
            op_names::DIV => binary(srcs, prev, out, |a, b| {
                if b.abs() < F::EPS {
                    F::ONE
                } else {
                    clamp(a / b)
                }
            }),
            op_names::SUM | op_names::LINEAR => activation(srcs, prev, out, clamp),
            op_names::SIGMOID => {
                activation(srcs, prev, out, |t| clamp(F::ONE / (F::ONE + (-t).exp())))
            }
            op_names::TANH => activation(srcs, prev, out, |t| clamp(t.tanh())),
            op_names::RELU => activation(srcs, prev, out, |t| clamp(t.max(F::ZERO))),
            op_names::LEAKY_RELU => activation(srcs, prev, out, |t| {
                let x = clamp(t);
                if x > F::ZERO { x } else { clamp(F::HALF * x) }
            }),
            _ => false,
        },
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Graph, GraphEvaluator, eval::EvalMut};

    fn check(graph: &Graph<Op<f32>>, rows: &[Vec<f32>]) {
        let row_refs: Vec<&[f32]> = rows.iter().map(|r| r.as_slice()).collect();
        let mut evaluator = GraphEvaluator::new(graph);
        let expected: Vec<Vec<f32>> = rows.iter().map(|r| evaluator.eval_mut(r)).collect();

        for mode in [ColMode::Generic, ColMode::Kernels] {
            let mut buf = ColBuf::default();
            let mut stats = ColStats::default();
            assert!(eval_columns(
                graph.as_ref(),
                &row_refs,
                &mut buf,
                mode,
                &mut stats
            ));
            for (r, want) in expected.iter().enumerate() {
                let got: Vec<f32> = buf
                    .output_slots()
                    .iter()
                    .map(|&s| buf.value(s, rows.len(), r))
                    .collect();
                let (got, want): (Vec<u32>, Vec<u32>) = (
                    got.iter().map(|v| v.to_bits()).collect(),
                    want.iter().map(|v| v.to_bits()).collect(),
                );
                assert_eq!(got, want, "mode {mode:?}, row {r}");
            }
        }
    }

    #[test]
    fn test_columns_match_row_evaluator() {
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

        let rows: Vec<Vec<f32>> = (0..17)
            .map(|i| vec![i as f32 * 0.3 - 2.0, 1.5 - i as f32 * 0.2])
            .collect();
        check(&graph, &rows);
    }

    #[test]
    fn test_recurrent_graph_is_not_columnable() {
        let mut graph = Graph::<Op<f32>>::default();
        let a = graph.insert(NodeType::Input, Op::var(0));
        let v = graph.insert(NodeType::Vertex, Op::sum());
        let out = graph.insert(NodeType::Output, Op::linear());
        graph.attach(a, v).attach(v, v).attach(v, out);

        let rows = [vec![1.0_f32]];
        let row_refs: Vec<&[f32]> = rows.iter().map(|r| r.as_slice()).collect();
        let mut buf = ColBuf::default();
        let mut stats = ColStats::default();
        assert!(!eval_columns(
            graph.as_ref(),
            &row_refs,
            &mut buf,
            ColMode::Kernels,
            &mut stats
        ));
    }
}
