# Changelog

All notable changes to Radiate are documented here. Format loosely follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project
adheres to semantic versioning.

## [1.3.0] — 2026-06-20

### Breaking

- **Removed the `radiate-pgm` crate** and its example. Probabilistic
  graphical model support is no longer shipped. The Rust `prelude` no longer
  re-exports `radiate_pgm::*`.
- **Removed the `radiate-expr` crate.** The metric expression DSL has moved
  into `radiate-core` under `stats::expression`, and is re-exported from the
  crate root (`Expr`, `SelectExpr`, `MetricQuery`, `Evaluate`,
  `expression::expr`). No separate dependency is required. The Rust `prelude`
  no longer re-exports `radiate_expr::*` — these types now flow through the
  `radiate-core` re-export instead, so `use radiate::prelude::*` continues to
  work without changes.
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
  module functions (`rd.select`, `rd.when`, `rd.lit`, `rd.every`,
  `rd.generation`, `rd.element`, `rd.pi_signal`, `rd.error_from`,
  `rd.is_converged`, `rd.stagnation`, `rd.is_stagnant`, `rd.p50/p95/p99`,
  `rd.quantile_stream`) are now classmethods on `Expr`
  (`rd.Expr.select(...)`, `rd.Expr.when(...)`, etc.), matching the library's
  `rd.Noun.factory` convention and keeping the `rd.*` namespace free of names
  that shadowed builtins. The redundant `rd.mean/min/max/stddev(metric)`
  shorthands were dropped — use `rd.Expr.select(metric).mean()`. The
  controller helper is now `rd.Expr.track(metric, target, ...)` (was the
  flat `adaptive_rate`). Expressions remain experimental; no back-compat
  aliases are kept.

### Added

- **Expression DSL — `compile()` pass.** Constant subtrees fold to literals
  and affine chains (`a*x + b`, including the common
  `(x - target) / target * gain + 1` controller pattern) collapse to a single
  fused `Affine` node. Idempotent.
- **Expression DSL — new operators.** `stagnation` / `is_stagnant` for
  patience-based plateau detection, `is_converged` for windowed convergence,
  `pi_signal` PI-control helper, and streaming P² quantiles (`p50`, `p95`,
  `p99`) with constant-memory online estimation.
- **Expression DSL (Python) — new methods.** `Expr.error(target)` and
  `Expr.quantile(q)` are now available on expression instances, alongside the
  classmethod constructors.
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

### Changed

- **Engine Iterator** the engine iterator is now a `runtime`. Instead of cloning the entire ecosystem every generation, the engine can now operate in a tight loop. The most common type of iterator methods used on the engine (`.last()`, `.take(n)`) are overwritten to work with the new design. That being said, if the user decides to use something like `.take_while(..)` or other more complex iterator methods, they will get the old behavior of cloning the entire ecosystem every generation. This is a pretty big change, but nothing should break from a user's perspective. This change should just make the engine more efficient.
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
- **NSGA III** - The NSGA III has been greatly simplified and optimized with a better niching technique. This is a pretty nice improvement over the previous implementation.
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