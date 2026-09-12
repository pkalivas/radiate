# Changelog

All notable changes to Radiate are documented here. Format loosely follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project
adheres to semantic versioning.

For all code examples and further explanations, refer to the [documentation](https://pkalivas.github.io/radiate/).

## [Unreleased]

`Rate` is fully replaced by the expression DSL, events/checkpointing/stopping move onto the engine builder, and the Python operator API is reorganized into namespaces (`Select.*`/`Cross.*`/`Mutate.*`/`Dist.*`/`Limit.*`/`Filter.*`/`Fitness.*`). Also new: a population-filter stage for stagnation recovery, adaptive species-count targeting, a `BitFlipMutator`, f64 support for GP graphs/trees, and three new TUI dashboard tabs.

### Breaking

- **`Rate` is replaced by `float` or [Expr](https://pkalivas.github.io/radiate/source/engine/expressions/).** (`impl Into<Expr>` is now used for conversion - minimal friction expected.) Every crossover/mutator now takes a plain `float` or `Expr` instead of a `Rate`. Python: `rd.Rate` is deleted. This was implemented in order to support full dynamic rates for _anything_ in `radiate`'s ecosystem.
- **[Events](https://pkalivas.github.io/radiate/source/events/) rewritten as typed pub/sub.** Implement `Handler<E>` for one event type (`EpochComplete`, `Improvement`, `EngineStart`/`Stop`, `LimitTriggered`, `Warning`, `CheckpointSaved`, ...) and register with `GeneticEngine::subscribe::<E>(handler)`. Python: new `rd.on_limit_triggered`/`on_log`/`on_checkpoint_saved` decorators; `event.index`/`event.event_type` are now attributes, not methods.
- **[Checkpointing](https://pkalivas.github.io/radiate/source/misc/checkpoint/#__tabbed_1_2) moved onto the builder.** Use `GeneticEngineBuilder::checkpoint(interval, path)` instead of `run()`-time checkpointing. Python: `Engine.write_checkpoint(path, interval, file_type="pkl")` replaces `run(checkpoint=...)`. Checkpoint pickle format also changed — **checkpoints written by 1.3.0 may not load**.
- **Metric-predicate stopping removed** — no more `Limit::Metric`/`Limit.metric(...)`. Use `Limit::Expr`/`Limit.expr(...)` instead. Expressions read directly from the `MetricSet` so this is _functionally_ equivalent.
- **Custom `Chromosome`/`Gene` implementors need updates.** Trait methods were re-split (bounds vs. init range, contiguous-storage methods moved to a new `ContiguousChromosome` sub-trait). No impact if you only use the built-in gene types.

### Changed 
- **GP ops/regression are generic over `f32`/`f64` now.** Calls like `Op::sigmoid()` may need `Op::<f32>::sigmoid()` if the type can't be inferred. Python `GraphCodec`/`TreeCodec` now take a `dtype` param.
- **Python: operators collapsed into namespaces.** `TournamentSelector(k=3)` → `Select.tournament(k=3)`, `BlendCrossover(...)` → `Cross.blend(...)`, `UniformMutator(...)` → `Mutate.uniform(...)`, `HammingDistance()` → `Dist.hamming()`, `ScoreLimit(...)` → `Limit.score(...)`. Old class names are no longer exported.
- **Python: `Engine.alters(...)` renamed `Engine.alter(...)`.**
- **Python: `Engine.run()` no longer takes `limits=`** See [limits](https://pkalivas.github.io/radiate/source/engine/limits/) (set limits on the builder) and `step_next()` is gone — iterate the engine directly (`for epoch in engine:`).
- **Python: `EngineConfig.max_species_age` default changed 20 → 25** to match Rust.
- **Python: install extras regrouped.** `[polars]`/`[pandas]`/`[numpy]`/`[matplotlib]` → `[data]` (numpy+pandas+polars) and `[plot]` (matplotlib); `[all]` unchanged.
- **Metric names renamed** see [default metrics](https://pkalivas.github.io/radiate/source/engine/metrics/#collection). Consistency across the engine and `radiate-gp` (e.g. `age.replace` → `replace.age`, `count.species` → `species.count`). See `docs/source/engine/metrics.md` for the full mapping.
- **Python: free-threaded wheels now actually target 3.14t** — CI had been building against 3.13t. This is out of `radiate`'s control and is a result of underlying maturin/pyo3 support.

### Added

- **Population filter pipeline stage** — new `UniqueScoreFilter` detects score-diversity collapse and replaces duplicates. `GeneticEngineBuilder::filter(...)` / Python `rd.Filter.unique_score(...)` + `Engine.filter(...)`.
- **Adaptive species-count targeting** — see [target species](https://pkalivas.github.io/radiate/source/diversity/species/#adaptive-thresholds). Set `target_species_count` to drive the speciation threshold toward a target instead of a fixed value. Python: `Engine.diversity(dist, threshold, target=...)`.
- **`BitFlipMutator`** for bit-string genomes. Python: `Mutate.bit_flip(rate=0.1)`.
- **`HealthMonitor` event handler** — auto-emits `Warning` events for stagnation, diversity collapse, and species collapse.
- **f64 support end-to-end for GP graphs/trees**, plus new ops: `Op.weight2`/`sign`/`reciprocal`/`gaussian`/`tooth`.
- **NumPy-native fitness & regression I/O** — custom Python fitness functions can return numpy arrays directly; regression accepts numpy arrays or lists.
- **`GraphMutator::target_size(size)`** — anti-bloat throttling once a graph reaches a target size.
- **Python: `Graph`/`Tree` pickle support** (`to_pickle`/`from_pickle`) and an `unchecked=True` fast path on `.eval()` that skips shape validation. `Graph.eval`/`Tree.eval` now also accept numpy arrays directly.
- **Python: `Expr.select(...)` methods** — `.mean()`/`.stddev()`/`.min()`/`.max()`/`.sum()`/`.slope()`/etc., plus `Expr.alias(name)`.
- **TUI: three new dashboard tabs** — "Improvements", "Front" (Pareto-front tracking), and "Events", plus a richer status bar and better table navigation keys.
- **New examples** — Bevy-based `flappy-bird` (Rust), CPPN image evolution `graph_art.py` (Python).

### Fixed

- **`GraphMutator` could pick duplicate source-node indices** for multi-arity ops — now deduplicated.
- Fixed a few panics from misaligned indices in graph/tree crossover.
- **`AnyValue`'s numeric → `Duration` cast now treats the source `f32` as seconds, not milliseconds** — durations reported via metrics were previously 1000x off.
- **LARGE BUG WITH CROSSOVER PARENT SELECTION** — previously, the engine could over select the same individual multiple times as a parent during crossover, leading to unexpected behavior and reduced genetic diversity. This has now been fixed to ensure a more even distribution of parent selection.

**For example and details please refer to the [user guide](https://pkalivas.github.io/radiate/) and API docs.**

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
