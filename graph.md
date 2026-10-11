# Graph evaluation: caching vs. column-wise evaluation

Investigation from 2026-10-11 on `feature/RD-LocalSearch.01` (base `b403d1c7`), looking for
where `Regression` spends its time on `GraphChromosome`s, using the iris-graph example.

**Short version:** caching the compiled graph saves at most about 4%, because compiling was
only about 4.4% of the profile to begin with. Evaluating column-wise (each node over every row
at once) instead of row by row cuts `step.evaluate` time by about 2.5× and the whole iris run
by about 2.1×, and the evolution is bit-for-bit identical.

All prototype code lives in a scratch copy of the repo. Nothing in the real repo was changed
apart from adding this file.

---

## 1. Background: what was already done

`HEAD` (`b403d1c7`) already has the **instruction-tape evaluator** in
`crates/radiate-gp/src/collections/graphs/eval.rs`:

- `GraphEvalCache::compile` walks `iter_topological()` once and turns each node into an
  `Instr { node, src_start, src_end }`. Its incoming edges are remapped to evaluation-order
  "slots" in one flat `srcs: Vec<u32>`.
- `eval_into_mut` walks contiguous arrays (`instrs`, `srcs`, `ops`) and never touches a
  `GraphNode`.
- Unreached nodes (e.g. in invalid graphs) get trailing slots that stay at `V::default()`.
- Cache reuse is keyed on a structural FNV fingerprint, not on node count.

`fitness.rs:52` still calls `GraphEvaluator::new(&input[0])` once per genotype, which compiles
the tape every time.

## 2. The caching stashes, and why they barely help

| Stash | What it does |
|---|---|
| `stash@{3}` (05:30) | Caches the compiled program on each `GraphChromosome` in a `OnceLock<Arc<GraphProgram>>`, cleared on every `&mut` path. Crossover and the op mutator write through `set_node_value`. Touches 7 files. |
| `stash@{2}`, `{1}`, `{0}` (06:07–06:24) | Iterations of one idea: a shared `Arc<Mutex<LruCache<u64, Arc<GraphStructure>>>>` inside `Regression`, keyed by a structural hash. `eval.rs` gets a `GraphStructure` that holds no ops. `{1}` has the hit-rate `println!` switched on; `{0}` comments it out and drops `node_type` from the hash. |
| `stash@{4}` | `mona_lisa.py` tweaks, not related. |

Why they don't do much:

- **There's little to save.** Building the evaluator (`iter_topological`, `is_recurrent`, the
  drop) was about 4.4% of profile samples, and the measured gain was about 4% (evaluate mean
  354.9 µs → 338.8 µs). The remaining cost is the interpreter loop running over every data row.
- **The LRU versions cost something on every batch:** hashing every genotype, two lock
  acquisitions, and building a `HashMap` and `Vec`. With `.parallel()` switched on, the shared
  `Mutex` would also make workers wait on each other.
- **The LRU versions have a correctness hazard.** `{0}` hashes edges only, but `compile`
  takes `output_slots` from `NodeType::Output`. A hash collision or a node-type difference
  means the wrong program is used silently; the only check is
  `debug_assert_eq!(node_count)`. If any version is kept, put `node_type` back in the hash.
- **`stash@{3}`'s `debug_assert`** recompiles on every `program()` call, so debug-build
  timings are misleading.

Worth keeping regardless: the iris example's `HashSet` → `BTreeSet` for `unique_labels`
(already in your working tree). With `HashSet`, the label → one-hot column mapping changed
from run to run, so seeded runs weren't comparable.

## 3. Column-wise evaluation

### Idea

Per graph, the row evaluator runs `nodes × rows` iterations of "gather inputs into scratch,
make an indirect `op.eval` call". With about 40 nodes and 113 rows that's about 4,500
indirect calls per graph per evaluation. The profile put about 44% of samples in that loop's
overhead and about 32% in the op math.

Column-wise, each node runs over **all rows at once**, writing to a slot-major buffer
`cols[slot * rows + r]`. That means:

- the dispatch happens once per node (about 40 per graph), not once per node per row;
- a kernel's inner loop is a plain loop over slices that the compiler can optimise.

### When it's exact

The row evaluator keeps `outputs` between rows; that state is how recurrent back edges read
the previous row's value. Column-wise evaluation is exactly equivalent **if and only if every
node only reads slots computed before it in the same row**:

```
for each instr at slot i: every src slot s < i
```

That excludes back edges, self loops, and reads of unreached (trailing) slots. Graphs that
fail the check fall back to the existing `GraphEvaluator`, so recurrent semantics are
unchanged. In practice every iris graph passed: the codec is `GraphCodec::directed`.

### Bit-exactness of kernels

Each kernel repeats the scalar op's float operations in the same order:

- `aggregate` sums 1–5 inputs left to right starting **from the first input**, and folds from
  `ZERO` above 5 inputs. `aggregate_into` copies this exactly (it matters for `-0.0`).
- Every `clamp` is in the same place, including `LeakyReLU`'s `clamp(HALF * clamp(x))`.
- `div`'s `abs() < EPS → ONE` branch is preserved.
- `exp`/`tanh` are the same scalar `F::exp`/`F::tanh` calls, with no vectorised approximation.
  A vectorised `exp` would be faster but not bit-exact, so it would have to be a separate,
  opt-in step.

Loss is computed by calling the same `Loss::calculate`, with a closure that copies row `r`'s
outputs from the columns, so the summation order of the loss is identical too.

### Modes, used to separate the two effects

- `ColMode::Generic`: column layout, but every node calls `op.eval` once per row (gathering
  from the columns). This measures the loop-order and layout change alone.
- `ColMode::Kernels`: column kernels for `var`, `const`, `w` (weight), `identity`,
  `add`/`sub`/`mul`/`div`, `sum`, `linear`, `sigmoid`, `tanh`, `relu`, `l_relu`. Every other
  op falls back to Generic for that node.

## 4. Results

### Snapshot microbenchmark

Real populations (100 graphs) were taken from a seeded (1000) iris run at several
generations. Each graph's loss over the 113 training rows was computed in all three modes.
Times are medians over 9 trials of 20 repetitions, with the mode order rotated between trials.

| Gen | Mean nodes | Columnable | Kernel-node share | Loss mismatches | Row | Col generic | Col kernels |
|---|---|---|---|---|---|---|---|
| 1 | 8.0 | 100/100 | 100% | 0 | 4.60 µs | 3.86 µs (1.19×) | 2.09 µs (2.20×) |
| 100 | 11.9 | 100/100 | 84% | 0 | 8.71 µs | 5.51 µs (1.58×) | 3.04 µs (2.87×) |
| 500 | 21.0 | 100/100 | 72% | 0 | 19.17 µs | 10.16 µs (1.89×) | 6.21 µs (3.09×) |
| 1000 | 30.0 | 100/100 | 54% | 0 | 22.80 µs | 14.76 µs (1.55×) | 9.99 µs (2.28×) |
| 2000 | 40.0 | 100/100 | 67% | 0 | 25.09 µs | 17.18 µs (1.46×) | 9.11 µs (2.75×) |

"Loss mismatches" compares `f32::to_bits()` of the loss across all three modes.

### End to end

iris-graph, seed 1000, fixed `.take(2000)` instead of the time/score stop, not parallel,
release build, 4 alternating runs per mode:

| Mode | step.evaluate total | step.evaluate mean | Run total |
|---|---|---|---|
| Row (current) | 1.50 / 1.58 / 1.55 / 1.52 s | ~385 µs | ~1.76 s |
| Column generic | 1.04 / 1.00 / 0.98 / 0.97 s | ~249 µs | ~1.22 s |
| Column kernels | 0.61 / 0.64 / 0.60 / 0.60 s | ~153 µs | **~0.82 s** |

All 12 runs ended on the same best score (`0.08070554`) with the same `count.evaluation` and
`unique.scores` means, and the printed final `Generation` plus train/test accuracy diffed
identical apart from `duration`. The evolutionary trajectory is unchanged.

## 5. Caveats and what landing it would take

1. **The column layout with no kernels could ship alone** (about 1.5× on evaluate). It needs no
   `Op` change and touches only `eval.rs` and `fitness.rs`.
2. **Kernels need a proper home.** The prototype matches on the op's name string
   (`op_names::SIGMOID`, …), which is a hack: a user-defined op that reuses a built-in name
   would silently get the wrong kernel. The design that fits is an optional column function
   carried by the built-in ops, e.g. on `Op::Fn` or as a kernel id assigned by the
   constructors in `math.rs`, so the evaluator dispatches on something the op owns.
3. **Memory.** The buffer is `nodes × rows` floats: about 18 KB for iris, but a large dataset
   would need processing in blocks of rows (e.g. 256–1024 rows), which keeps it in cache too.
4. **More kernels add more.** The ops that still fall back (`exp`, `sin`, `cos`, `max`, `min`,
   `prod`, `gaussian`, `neg`, `abs`, …) are cheap to add in the same style.
5. **Compile is still per call** in both paths; that's the remaining ~4% the caching stashes
   targeted. Once evaluation is 2.5× faster, compiling becomes a larger share, so it may be
   worth revisiting caching afterwards with a fresh profile.
6. The prototype allocates `rows: Vec<&[F]>` and one `ColBuf` per batch. That's negligible
   here, but the final version should keep them alongside the existing loss buffer.

## 6. Code

The scratch copy is at
`/private/tmp/claude-501/-Users-peter-Dev-radiate/e9c6f510-6c06-44dc-8c1e-c5a73f02eb6c/scratchpad/col/radiate`
(built with `CARGO_TARGET_DIR=../target`).

### 6.1 `crates/radiate-gp/src/collections/graphs/eval.rs` (appended to the end of HEAD's file)

```rust
/// PROTOTYPE: column-wise evaluation of acyclic graphs. Each node runs over every row of the
/// data set at once, writing into a slot-major buffer (`cols[slot * rows + r]`).
pub mod column {
    use super::{GraphEvalCache, GraphNode};
    use crate::node::Node;
    use crate::{
        Eval, Op,
        ops::{OpFloat, op_names},
    };

    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub enum ColMode {
        /// Column layout, but every node calls `op.eval` once per row.
        Generic,
        /// Column layout with per-op kernels for the common ops; others fall back to Generic.
        Kernels,
    }

    #[derive(Default)]
    pub struct ColStats {
        pub graphs: usize,
        pub columnable: usize,
        pub kernel_nodes: usize,
        pub fallback_nodes: usize,
    }

    #[derive(Default)]
    pub struct ColBuf<F> {
        cols: Vec<F>,
        scratch: Vec<F>,
        srcs_tmp: Vec<u32>,
    }

    #[inline(always)]
    fn clamp<F: OpFloat>(value: F) -> F {
        if value.is_nan() {
            return F::ZERO;
        }
        value.clamp(-F::MAX_VALUE, F::MAX_VALUE)
    }

    /// Evaluates `nodes` over all `rows`. Returns `false` (and does nothing useful) when the
    /// graph isn't safely columnable, i.e. some node reads a slot that isn't computed before
    /// it in the same row (back edges, self loops, unreached nodes). Those graphs depend on
    /// row-to-row state and must use the row evaluator.
    ///
    /// On success, output `j` for row `r` is at `out_cols()[j][r]` via [ColBuf::output].
    pub fn eval_columns<F: OpFloat>(
        nodes: &[GraphNode<Op<F>>],
        rows: &[&[F]],
        buf: &mut ColBuf<F>,
        mode: ColMode,
        stats: &mut ColStats,
    ) -> Option<Vec<u32>> {
        let tape: GraphEvalCache<F> = GraphEvalCache::compile(nodes);
        stats.graphs += 1;

        for (slot, ins) in tape.instrs.iter().enumerate() {
            for &s in &tape.srcs[ins.src_start as usize..ins.src_end as usize] {
                if s as usize >= slot {
                    return None;
                }
            }
        }
        stats.columnable += 1;

        let n_rows = rows.len();
        let n_slots = nodes.len();
        buf.cols.clear();
        buf.cols.resize(n_slots * n_rows, F::default());
        if buf.scratch.len() < tape.scratch.len().max(1) {
            buf.scratch.resize(tape.scratch.len().max(1), F::default());
        }

        for (slot, ins) in tape.instrs.iter().enumerate() {
            let op = nodes[ins.node as usize].value();
            buf.srcs_tmp.clear();
            buf.srcs_tmp
                .extend_from_slice(&tape.srcs[ins.src_start as usize..ins.src_end as usize]);

            let (prev, rest) = buf.cols.split_at_mut(slot * n_rows);
            let out = &mut rest[..n_rows];
            let srcs = &buf.srcs_tmp;

            let done = mode == ColMode::Kernels && kernel(op, srcs, prev, rows, out);
            if done {
                stats.kernel_nodes += 1;
                continue;
            }
            stats.fallback_nodes += 1;

            if srcs.is_empty() {
                for (r, o) in out.iter_mut().enumerate() {
                    *o = op.eval(rows[r]);
                }
            } else {
                let scratch = &mut buf.scratch[..srcs.len()];
                for (r, o) in out.iter_mut().enumerate() {
                    for (d, &s) in scratch.iter_mut().zip(srcs.iter()) {
                        *d = prev[s as usize * n_rows + r];
                    }
                    *o = op.eval(scratch);
                }
            }
        }

        Some(tape.output_slots)
    }

    impl<F: Copy> ColBuf<F> {
        #[inline]
        pub fn value(&self, slot: u32, n_rows: usize, r: usize) -> F {
            self.cols[slot as usize * n_rows + r]
        }
    }

    #[inline(always)]
    fn col<F>(prev: &[F], s: u32, n: usize) -> &[F] {
        &prev[s as usize * n..s as usize * n + n]
    }

    /// Sum of the source columns into `out`, matching `aggregate` bit-for-bit: 1..=5 inputs
    /// sum left to right from the first input, more than 5 fold from zero.
    #[inline(always)]
    fn aggregate_into<F: OpFloat>(srcs: &[u32], prev: &[F], out: &mut [F]) {
        let n = out.len();
        match srcs.len() {
            0 => out.fill(F::ZERO),
            k => {
                if k <= 5 {
                    out.copy_from_slice(col(prev, srcs[0], n));
                } else {
                    out.fill(F::ZERO);
                    let c = col(prev, srcs[0], n);
                    for (o, &x) in out.iter_mut().zip(c) {
                        *o = *o + x;
                    }
                }
                for &s in &srcs[1..] {
                    let c = col(prev, s, n);
                    for (o, &x) in out.iter_mut().zip(c) {
                        *o = *o + x;
                    }
                }
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
    fn map_in_place<F: OpFloat>(out: &mut [F], f: impl Fn(F) -> F) {
        for o in out.iter_mut() {
            *o = f(*o);
        }
    }

    /// Column kernel for `op`, or `false` if there isn't one (caller falls back).
    fn kernel<F: OpFloat>(op: &Op<F>, srcs: &[u32], prev: &[F], rows: &[&[F]], out: &mut [F]) -> bool {
        let n = out.len();
        match op {
            Op::Var(_, index, _) if srcs.is_empty() => {
                let i = *index;
                for (o, row) in out.iter_mut().zip(rows) {
                    *o = row[i];
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
            Op::Fn(name, _, _) => {
                let name: &str = name;
                match name {
                    op_names::IDENTITY if !srcs.is_empty() => {
                        out.copy_from_slice(col(prev, srcs[0], n));
                        true
                    }
                    op_names::ADD => binary(srcs, prev, out, |a, b| clamp(a + b)),
                    op_names::SUB => binary(srcs, prev, out, |a, b| clamp(a - b)),
                    op_names::MUL => binary(srcs, prev, out, |a, b| clamp(a * b)),
                    op_names::DIV => binary(srcs, prev, out, |a, b| {
                        if b.abs() < F::EPS { F::ONE } else { clamp(a / b) }
                    }),
                    op_names::SUM | op_names::LINEAR => {
                        aggregate_into(srcs, prev, out);
                        map_in_place(out, clamp);
                        true
                    }
                    op_names::SIGMOID => {
                        aggregate_into(srcs, prev, out);
                        map_in_place(out, |t| clamp(F::ONE / (F::ONE + (-t).exp())));
                        true
                    }
                    op_names::TANH => {
                        aggregate_into(srcs, prev, out);
                        map_in_place(out, |t| clamp(t.tanh()));
                        true
                    }
                    op_names::RELU => {
                        aggregate_into(srcs, prev, out);
                        map_in_place(out, |t| clamp(t.max(F::ZERO)));
                        true
                    }
                    op_names::LEAKY_RELU => {
                        aggregate_into(srcs, prev, out);
                        map_in_place(out, |t| {
                            let x = clamp(t);
                            if x > F::ZERO { x } else { clamp(F::HALF * x) }
                        });
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }
}
```

### 6.2 `crates/radiate-gp/src/collections/graphs/mod.rs`

```diff
-pub use eval::{GraphEvalCache, GraphEvaluator};
+pub use eval::{GraphEvalCache, GraphEvaluator, column};
```

### 6.3 `crates/radiate-gp/src/regression/fitness.rs` (diff vs HEAD)

`RADIATE_COL=0|1|2` picks row / column generic / column kernels for the engine path;
`Regression::graph_loss` exposes the same switch for the benchmark.

```diff
--- /Users/peter/Dev/radiate/crates/radiate-gp/src/regression/fitness.rs	2026-10-11 06:24:37
+++ crates/radiate-gp/src/regression/fitness.rs	2026-10-11 06:38:28
@@ -1,6 +1,9 @@
 use super::{DataSet, Loss};
 use crate::{GraphChromosome, GraphEvaluator, Op, TreeChromosome, eval::EvalIntoMut, ops::OpFloat};
 use radiate_core::{BatchFitnessFunction, Genotype, Score};
+use crate::GraphNode;
+use crate::graphs::column::{ColBuf, ColMode, ColStats, eval_columns};
+use std::sync::OnceLock;
 
 #[derive(Clone)]
 pub struct Regression<F: OpFloat> {
@@ -48,9 +51,12 @@
     fn evaluate(&self, inputs: &[&'a Genotype<GraphChromosome<Op<F>>>]) -> Vec<F> {
         self.with_buff(move |buf| {
             let mut results = Vec::with_capacity(inputs.len());
+            let mode = col_mode();
+            let rows: Vec<&[F]> = self.data_set.iter().map(|(x, _)| x).collect();
+            let mut cbuf = ColBuf::default();
+            let mut stats = ColStats::default();
             for input in inputs.iter() {
-                let mut evaluator = GraphEvaluator::new(&input[0]);
-                results.push(self.calc_with_buf(&mut evaluator, buf));
+                results.push(self.graph_loss_buf(input[0].as_ref(), mode, &rows, buf, &mut cbuf, &mut stats));
             }
 
             results
@@ -58,6 +64,56 @@
     }
 }
 
+/// PROTOTYPE switch: RADIATE_COL=0 (row, default) | 1 (column generic) | 2 (column kernels).
+pub fn col_mode() -> Option<ColMode> {
+    static MODE: OnceLock<Option<ColMode>> = OnceLock::new();
+    *MODE.get_or_init(|| match std::env::var("RADIATE_COL").as_deref() {
+        Ok("1") => Some(ColMode::Generic),
+        Ok("2") => Some(ColMode::Kernels),
+        _ => None,
+    })
+}
+
+impl<F: OpFloat> Regression<F> {
+    /// PROTOTYPE: loss of one graph using the given evaluation mode (None = row evaluator).
+    pub fn graph_loss(
+        &self,
+        nodes: &[GraphNode<Op<F>>],
+        mode: Option<ColMode>,
+        cbuf: &mut ColBuf<F>,
+        stats: &mut ColStats,
+    ) -> F {
+        let rows: Vec<&[F]> = self.data_set.iter().map(|(x, _)| x).collect();
+        self.with_buff(|buf| self.graph_loss_buf(nodes, mode, &rows, buf, cbuf, stats))
+    }
+
+    fn graph_loss_buf(
+        &self,
+        nodes: &[GraphNode<Op<F>>],
+        mode: Option<ColMode>,
+        rows: &[&[F]],
+        buf: &mut [F],
+        cbuf: &mut ColBuf<F>,
+        stats: &mut ColStats,
+    ) -> F {
+        if let Some(mode) = mode {
+            if let Some(out_slots) = eval_columns(nodes, rows, cbuf, mode, stats) {
+                let n = rows.len();
+                let mut r = 0;
+                return self.loss.calculate(&self.data_set, buf, |_, y| {
+                    for (d, &slot) in y.iter_mut().zip(out_slots.iter()) {
+                        *d = cbuf.value(slot, n, r);
+                    }
+                    r += 1;
+                });
+            }
+        }
+
+        let mut evaluator = GraphEvaluator::new(&nodes);
+        self.calc_with_buf(&mut evaluator, buf)
+    }
+}
+
 impl<'a, F> BatchFitnessFunction<&'a Genotype<TreeChromosome<Op<F>>>, F> for Regression<F>
 where
     F: OpFloat + Into<Score>,
```

### 6.4 `examples/rust/graphs/iris-graph/src/bin/colbench.rs` (snapshot benchmark)

```rust
use radiate::*;
use radiate::graphs::column::{ColBuf, ColMode, ColStats};
use std::collections::BTreeSet;
use std::hint::black_box;
use std::time::Instant;

const CHECKPOINTS: [usize; 5] = [1, 100, 500, 1000, 2000];

fn main() {
    random_provider::seed(1000);
    let (train, _test) = load_iris_dataset().shuffle().standardize().split(0.75);

    let ops = ops::all_ops();
    let store = vec![
        (NodeType::Input, (0..4).map(Op::var).collect()),
        (NodeType::Edge, vec![Op::identity(), Op::weight()]),
        (NodeType::Vertex, ops.clone()),
        (NodeType::Output, vec![Op::sigmoid()]),
    ];
    let codec = GraphCodec::directed(4, 4, store);
    let regression = Regression::new(train.clone(), Loss::MSE);

    let engine = GeneticEngine::builder()
        .codec(codec)
        .minimizing()
        .fitness_fn(regression.clone())
        .offspring_fraction(0.92)
        .replace_strategy(GraphReplacement)
        .offspring_selector(BoltzmannSelector::new(4.0))
        .alter(alters!(
            GraphCrossover::new(0.5, 0.5),
            OperationMutator::new(0.02, 0.05),
            GraphMutator::new(0.008, 0.002)
        ))
        .build();

    let mut snaps: Vec<(usize, Vec<GraphChromosome<Op<f32>>>)> = Vec::new();
    for generation in engine.iter().take(*CHECKPOINTS.last().unwrap()) {
        let idx = generation.index();
        if CHECKPOINTS.contains(&idx) {
            let pop = generation.population().iter().map(|p| p.genotype()[0].clone()).collect();
            snaps.push((idx, pop));
        }
    }

    let modes: [(&str, Option<ColMode>); 3] = [
        ("row", None),
        ("col-generic", Some(ColMode::Generic)),
        ("col-kernels", Some(ColMode::Kernels)),
    ];
    let mut cbuf = ColBuf::default();

    println!("rows in train set: {}", train.len());
    for (gen_idx, graphs) in &snaps {
        // structure + exactness
        let mut stats = ColStats::default();
        let mut mismatches = 0;
        let mut node_total = 0;
        for g in graphs {
            node_total += g.as_ref().len();
            let base = regression.graph_loss(g.as_ref(), None, &mut cbuf, &mut ColStats::default());
            let a = regression.graph_loss(g.as_ref(), Some(ColMode::Generic), &mut cbuf, &mut ColStats::default());
            let b = regression.graph_loss(g.as_ref(), Some(ColMode::Kernels), &mut cbuf, &mut stats);
            if base.to_bits() != a.to_bits() || base.to_bits() != b.to_bits() {
                mismatches += 1;
                if mismatches <= 3 {
                    println!("  MISMATCH row={base:e} generic={a:e} kernels={b:e}");
                }
            }
        }
        let kn = stats.kernel_nodes as f64 / (stats.kernel_nodes + stats.fallback_nodes).max(1) as f64;
        println!(
            "gen {gen_idx:>5}: {} graphs, mean nodes {:.1}, columnable {}/{}, kernel-node share {:.0}%, loss mismatches {}",
            graphs.len(),
            node_total as f64 / graphs.len() as f64,
            stats.columnable,
            stats.graphs,
            kn * 100.0,
            mismatches
        );

        // timing: alternate modes across trials, take median per mode
        let reps = 20;
        let trials = 9;
        let mut samples: Vec<Vec<f64>> = vec![Vec::new(); modes.len()];
        for t in 0..trials {
            for k in 0..modes.len() {
                let m = (k + t) % modes.len();
                let mode = modes[m].1;
                let mut st = ColStats::default();
                let start = Instant::now();
                for _ in 0..reps {
                    for g in graphs {
                        black_box(regression.graph_loss(black_box(g.as_ref()), mode, &mut cbuf, &mut st));
                    }
                }
                let per = start.elapsed().as_nanos() as f64 / (reps * graphs.len()) as f64;
                samples[m].push(per);
            }
        }
        let med = |v: &mut Vec<f64>| {
            v.sort_by(|a, b| a.partial_cmp(b).unwrap());
            v[v.len() / 2]
        };
        let row = med(&mut samples[0]);
        for (m, (name, _)) in modes.iter().enumerate() {
            let x = med(&mut samples[m]);
            println!("    {name:<12} {:>9.2} us/graph   ({:.2}x vs row)", x / 1000.0, row / x);
        }
    }
}

// fn load_iris_dataset() -> DataSet<f32> { ... same as iris-graph/src/main.rs ... }
```

### 6.5 End-to-end runs

In the scratch copy, the iris-graph `main.rs` has `.until(...)` replaced with `.take(2000)`.
Then:

```sh
cargo build --release -p iris-graph
for i in 1 2 3 4; do for m in 0 1 2; do
  RADIATE_COL=$m ./target/release/iris-graph > r_${m}_$i.txt 2>&1
done; done
grep '^step.evaluate.time' r_*.txt
```
