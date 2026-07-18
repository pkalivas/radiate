# Changelog

All notable changes to Radiate are documented here. Format loosely follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project
adheres to semantic versioning.

## [Unreleased]

This release finishes the `Rate` → expression-DSL migration started in `1.3.0` — the `Rate` type is now gone entirely, in both Rust and Python, replaced by `Expr`/`RateSet` everywhere a schedule is accepted. It also splits the expression DSL back out into its own `radiate-expr` crate, adds a population-filter pipeline stage (with an initial `UniqueScoreFilter` for stagnation recovery), adds adaptive species-count targeting, generalizes GP ops/regression over `f32`/`f64`, and restructures a large chunk of the Python operator API around static-method namespaces (`Select.*`, `Cross.*`, `Mutate.*`, `Dist.*`, `Limit.*`, `Filter.*`, `Fitness.*`). The TUI also picked up two new dashboard tabs.

### Breaking

- **`Rate` is gone.** The `Rate` enum (`Fixed`/`Linear`/`Exponential`/`Cyclical`/`Stepwise`/`Expr`) has been removed from `radiate-core` entirely; every crossover and mutator now builds its schedule from `Expr`/`RateSet` instead (`Crossover::rate()`/`Mutate::rate()` → `rates() -> RateSet`, constructors take `impl Into<Expr>`). `AlterContext::new()` gained an `internal_rates: &[f32]` parameter for multi-parameter alterers. `Alterer::apply` now returns `RadiateResult<()>`. On the Python side, `rd.Rate` and `PyRate` are deleted outright — every alterer parameter that used to take a `Rate` now takes a plain `float` or `Expr`.
- **Expression DSL extracted into its own crate, `radiate-expr`.** It had been folded into `radiate-core/src/stats/expression/` as of `1.3.0`; that's been reversed — the DSL now lives in `crates/radiate-expr`, with `radiate-core` depending on it and re-exporting via `pub use radiate_expr::*`, so `radiate::prelude` imports are unaffected. The one real API break: `GeneticEngineBuilder::register_metrics(Vec<impl Into<MetricQuery>>)` is renamed to `metrics(impl Into<ExprSet>)`.
- **`Gene`/`Chromosome` trait surface changed.** `BoundedGene::{min,max,bounds}` → `{init_min,init_max,init_range,bound_min,bound_max,bound_range}`, splitting initialization range from hard bounds. `ArithmeticGene` is gone, merged into a real `NumericGene` trait (`Add`/`Sub`/`Mul`/`Div` + `safe_add`/`sub`/`mul`/`div`). `Chromosome::get`/`get_mut` now return `Option<&Gene>` instead of panicking; new `apply_paired()` replaces ad-hoc `zip_mut` loops.
- **GP ops and regression are now generic over float precision.** `Op<f32>` → `Op<F: OpFloat>` (`f32` or `f64`), propagated through `math_ops()`/`activation_ops()`/`all_ops()`, `Regression<F>`, `Accuracy<F>`, `DataSet<F>`, `Loss::calc<F>`, and `NeatDistance`. Calls like `Op::sigmoid()` that previously inferred `f32` may now need `Op::<f32>::sigmoid()` in ambiguous contexts. On the Python side, `_all_ops`/`_activation_ops`/`_edge_ops` now require a `dtype` argument, and `GraphCodec`/`TreeCodec` take a `dtype: "float32" | "float64"` param.
- **Python: operator classes collapsed into namespace statics.** `TournamentSelector(k=3)` → `Select.tournament(k=3)`, `BlendCrossover(...)` → `Cross.blend(...)`, `UniformMutator(...)` → `Mutate.uniform(...)`, `HammingDistance()` → `Dist.hamming()`, `ScoreLimit(...)`/`GenerationsLimit(...)` → `Limit.score(...)`/`Limit.generations(...)`. None of the old class names are exported from `radiate/__init__.py` anymore. The `rd.dsl` shim that used to wrap these is gone; the logic moved directly into `radiate/operators/*.py`.
- **Python: fitness classes consolidated onto `rd.Fitness`.** `radiate/fitness/` is deleted; `Regression(features, targets, ...)` → `Fitness.regression(dtype, features, targets, ...)` (note: `dtype` is now a required first positional arg), `NoveltySearch(...)` → `Fitness.novelty(...)`, and `BatchFitness` is gone in favor of an `is_batch`/`batch` flag on `Fitness.custom(...)`.
- **Python: DSL modules moved under `radiate.dsl`.** `radiate.expr` → `radiate.dsl.expr`, `radiate.dtype` → `radiate.dsl.dtype`, `radiate.fitness.loss` → `radiate.dsl.loss`. Top-level `rd.Expr`/`rd.Float32`/`rd.MSE` re-exports are unchanged; direct submodule imports break.
- **Python: engine run/iteration API reshaped.** `PyEngine.run()` no longer accepts a `limits` argument (set limits on the builder instead); `step_next()` is gone, replaced by the native iterator protocol (`for epoch in engine: ...`). Both now release the GIL during execution, so an engine run no longer blocks other Python threads.
- **Python: `Graph.eval`/`Tree.eval`** now accept numpy arrays or lists directly instead of `Vec[float] | Vec[Vec[float]]`.
- **Python: `EngineBuilder.inputs`** changed from a method to a property (`builder.inputs()` → `builder.inputs`).
- **Python: `OpsConfig` removed** from the public API.
- **Python: package extras renamed.** `pip install radiate[polars]`/`[pandas]`/`[numpy]`/`[matplotlib]` are gone — regrouped into `radiate[data]` (numpy + pandas + polars), `radiate[plot]` (matplotlib), and `radiate[torch]`; `radiate[all]` still pulls everything.
- **Checkpoint pickle I/O reworked** to route through `PyGeneration::to_pickle`/`from_pickle` instead of the crate's own writer — pickle checkpoints written by `1.3.0` may not load cleanly; worth a compatibility check before release.

### Added

- **Population filter pipeline stage.** New `EcosystemFilter` trait runs after the existing invalid/age replacement (`radiate-engines/src/steps/filter.rs`), wired in via `GeneticEngineBuilder::filter()`/`filters()`. Ships with `UniqueScoreFilter`, which detects population score-diversity collapse (via `Expr::select(...).stagnation(...)`) and replaces duplicates. Exposed to Python as `rd.Filter.unique_score(threshold, max_stagnation)` + `Engine.filter(...)`.
- **Adaptive species-count targeting.** Setting `target_species_count` now drives the speciation threshold as an `Expr`/`RateSet` toward a target count instead of a fixed threshold. Python: `Engine.diversity(dist, threshold, target=...)`.
- **Expression DSL: `.stagnation(epsilon)`** — detects a fitness plateau (consecutive generations under an epsilon delta).
- **f64 support end-to-end for GP graphs/trees**, via the `Op<F: OpFloat>` generalization above.
- **NumPy-native fitness & regression I/O** — custom Python fitness functions can return numpy score arrays directly; regression fitness/accuracy accept numpy arrays or plain lists for features/targets.
- **`GraphMutator::target_size(size)`** — anti-bloat throttling that reduces vertex/edge mutation rates once a graph reaches a target size.
- **`radiate-ui`: two new dashboard tabs.** "Improvements" (single-objective) shows a scrollable improvement-event log plus a delta bar chart; "Front" (multi-objective) shows Pareto-front additions/removals/comparisons alongside front-size/entropy line charts. Also: a paused-start mode (`radiate::ui((engine, true))`), richer status bar (best-vs-current score, stagnation count, trend arrows on diversity/entropy), and new table navigation keybindings (PageUp/PageDown, Home/End).
- **Python: `Generation.gene_type()`.**

### Changed

- **Metric names renamed** for consistency, in both `radiate-gp` (`mutate.graph.invalid.*` → `mutator.graph.*`, `mutate.operation.*` → `mutator.op.*`) and the engine (`age.replace` → `replace.age`, `size.genome` → `genome.size`, `new.front` → `front.additions`, `invalid.front` → `front.removals`, `count.species` → `species.count`, and others). See `docs/source/engine/metrics.md` for the full list.
- **Python: free-threaded builds now target 3.14t.** CI was still building free-threaded wheels against `python3.13t` despite `1.3.0`'s changelog claiming 3.14/t support — the publish workflow now actually targets 3.14t (the abi3 free-threaded stable ABI only exists from CPython 3.14).

### Docs

- **`docs/source/alters/rate.md` rewritten** for the `Expr`-based rate model — this finishes the doc that was flagged as stale after the `1.3.0` Rate→Expr migration. Schedule names updated (`Fixed`/`Linear`/`Stepwise`/`Sine`/`Triangular`/`Exponential` → `Constant`/`Linear ramp`/`Stepped`/`Periodic`/`Exponential decay`/`Metric-driven`).
- **`docs/source/diversity/species.md`** — new "Target Species Count" section.
- **`docs/source/engine/metrics.md`** — updated for the renamed metrics above, plus new metrics (`scores.best`, `scores.evenness`, `scores.gini`, `genome.size.score.corr`).
- Added the missing `1.3.0` entry to `docs/source/releases.md`.

### Fixed

- **`GraphMutator` could select duplicate source-node indices** when inserting a multi-arity op (`Arity::Exact(n>1)`); node selection is now deduplicated.
- Defensive `Option`-based node lookups in graph/tree crossover, removing a few panic-on-misaligned-index paths.

**For example and details please refer to the user guide and API docs.**

## [1.3.0] — 2026-06-20

Radiate has reached `1.3.0`! This release includes a major refactor of the engine's iteration model, a new expression DSL pass and operators, a substantial NSGA III simplification and optimization, a NEAT implementation refactor, and various other improvements and cleanups across the codebase. The `engine` is now _much_ more efficient, and the new expression DSL features should make it easier to implement complex adaptive behaviors without custom code. The user guide has been rebuilt around testable snippets, and a new guide on diversity and speciation has been added. Check below!

**User guide has been updated and expanded, if you haven't looked at it in a while, check it out!**

### Breaking

- **Removed the `radiate-pgm` crate** and its example. Probabilistic
  graphical model support is no longer shipped. The Rust `prelude` no longer
  re-exports `radiate_pgm::*`.
    * I don't think this was really used at all and it never really reached more than an infancy state. If PGMs are a desired feature, they can be reintroduced in the future with a more focused scope and better design - please open an issue if you'd like to see them.
- **Removed the `radiate-expr` crate.** The metric expression DSL has moved
  into `radiate-core` under `stats::expression`, and the core types are
  re-exported from the `radiate-core` crate root (`Expr`, `SelectExpr`,
  `MetricQuery`, `Evaluate`), and are also available through
  `radiate::prelude` (so `use radiate::prelude::*;` brings `Expr` into scope).
  No separate dependency is required, and the `prelude` no longer re-exports
  `radiate_expr::*`.
- **Removed the `lineage` module** from `radiate-core`. `Lineage`,
  `LineageEvent`, and `LineageUpdate` are no longer part of the public API.
- **Removed the `Field` struct** from `radiate-utils` (and its re-export).
  `AnyValue` / `DataType` naming was cleaned up at the same time. If you were
  constructing `Field` values directly, use `DataType` / `AnyValue` instead.
- **Python: run options restructured.** `EngineUi` has been removed in favor
  of a `RunParam` hierarchy — `LogParam`, `CheckpointParam`, and `UiParam`.
  The `bool` shorthands on `run(...)` are unchanged (`engine.run(ui=True,
  log=True)` still works); only the typed option objects were renamed. Code
  that imported or constructed `EngineUi` must switch to `UiParam`.
- **Python: expression DSL constructors moved onto `rd.Expr`.** The flat
  module functions that previously lived on `rd` — `rd.metric`, `rd.when`,
  `rd.lit`, `rd.element`, `rd.every`, and `rd.generation` — are now
  classmethods on `Expr`, and `rd.metric(...)` is renamed to
  `rd.Expr.select(...)` (so: `rd.Expr.when(...)`, `rd.Expr.lit(...)`,
  `rd.Expr.element(...)`, etc.). This matches the library's
  `rd.Noun.factory` convention and keeps the `rd.*` namespace free of names
  that shadowed builtins. The redundant `rd.mean/min/max/stddev(metric)`
  shorthands were dropped — use `rd.Expr.select(metric).mean()`. Expressions
  remain experimental; no back-compat aliases are kept.
- **Metric names have changed**. The names of certain metrics have changed to be more consistent and intuitive. Run a quick engine and print out the `metrics.dashboard()` to see new names or checkout the user guide.

### Added

- **Expression DSL — `compile()` pass.** Constant subtrees fold to literals
  and affine chains (`a*x + b`, including the common
  `(x - target) / target * gain + 1` controller pattern) collapse to a single
  fused `Affine` node. Idempotent.
- **Expression DSL — new operators.** `stagnation` / `is_stagnant` for
  patience-based plateau detection, `is_converged` for windowed convergence,
  and a streaming P² quantile (`.quantile(q)`) with constant-memory online
  estimation.
- **Expression DSL (Python) — new constructors & methods.** New `Expr`
  classmethods `Expr.stagnation(metric, ...)` and
  `Expr.is_stagnant(metric, patience, ...)`, plus instance methods
  `Expr.error(target)` and `Expr.quantile(q)`.
- **Python: `Species` is now inspectable.** New accessors expose speciation
  results to Python — `population()`, `mascot()`, `generation()`,
  `stagnation()`, and `score()`.
- **Python: `Graph.from_chromosome(chromosome)`** classmethod for building a
  `Graph` directly from a chromosome.
- **Python: `Dict` nested dtype** and a `Population.population(size)` helper.
- **New examples** — `neat-graph` (Rust) demonstrating NEAT-style
  neuroevolution, plus reorganized Python examples (`hello_world`,
  `mona_lisa`, `playground`).
- **Workspace-wide re-exports from `radiate-core`** — `AnyValue`, `DataType`,
  `SmallStr`, `dtype`, `dtype_names`, and `value` are now re-exported from
  `radiate-core` so downstream crates no longer need to depend on
  `radiate-utils` directly for these types.
- **Python 3.14/t** support. Radiate now supports Python 3.14/t and has **dropped** support for python 3.13t. Maturin/PyO3 now requires Python 3.14 or later to build free-threaded bindings. Single threaded bindings will still work for lower versions.

### Changed

- **Engine Iterator** the engine iterator is now a `runtime`. Instead of cloning the entire ecosystem every generation, the engine can now operate in a tight loop. The most common type of iterator methods used on the engine (`.last()`, `.take(n)`) are overwritten to work with the new design. That being said, if the user decides to use something like `.take_while(..)` or other more complex iterator methods, they will get the old behavior of cloning the entire ecosystem every generation. This is a pretty big change, but nothing should break from a user's perspective. This change should just make the engine _much_ more efficient.
- **Recombination is substantially faster.** Survivor and offspring
  construction now share a single descending walk over the union of
  selected indices, emitting one `swap_remove` move plus `(total - 1)`
  clones per unique source rather than cloning every survivor and offspring
  independently. The species path applies the same optimization within
  each species' sub-population. In practice ~20–50% fewer `Phenotype`
  clones per generation.
- **Metric subsystem cleanup.** Stale fixture data and brittle snapshot
  tests removed; `MetricView` is leaner and now covered by direct unit
  tests.
- **UI refactor.** Panel state moved off the `PanelId` dispatch model into
  a cleaner per-panel ownership scheme.
- **NSGA III** - The NSGA III has been greatly simplified and optimized with a better niching technique. This is a pretty nice mathematical and speed improvement over the previous implementation.
- **NEAT** - the Graph's NEAT implementation has been refactored to be more true to the original paper. The `neat-graph` example demonstrates this new implementation.

### Docs

- **User guide rebuilt around testable snippets.** Every code sample is now
  extracted from a real source file under `docs/source/src/{python,rust}/...`
  via `pymdownx.snippets` (`check_paths: true`), and `mkdocs build --strict`
  validates that the referenced snippets exist and compile/run. Docs samples
  can no longer silently drift from the API.
- **New diversity / speciation guide** (`source/diversity/species.md`) plus
  expanded Rust + Python parity pages across genome, alters, selectors, GP,
  objectives, events, and executors.
- **Removed PGM documentation** (`source/gp/pgm.md`) alongside the
  `radiate-pgm` crate removal.

### Fixed

- Various clippy and correctness cleanups across the alters, selectors,
  and engine crates (no behavior changes intended; flagged here only
  because several touched hot paths).

**For example and details please refer to the user guide and API docs.**