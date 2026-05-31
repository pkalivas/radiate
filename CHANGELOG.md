# Changelog

All notable changes to Radiate are documented here. Format loosely follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project
adheres to semantic versioning.

## [Unreleased] — successor to 1.2.22

### Breaking

- **Removed the `radiate-pgm` crate** and its example. Probabilistic
  graphical model support is no longer shipped.
- **Removed the `radiate-expr` crate.** The metric expression DSL has moved
  into `radiate-core` under `stats::expression`, and is re-exported from the
  crate root (`Expr`, `SelectExpr`, `MetricQuery`, `Evaluate`,
  `expression::expr`). No separate dependency is required.
- **Removed the `lineage` module** from `radiate-core`. `Lineage`,
  `LineageEvent`, and `LineageUpdate` are no longer part of the public API.
- **Python: expression DSL constructors moved onto `rd.Expr`.** The flat
  module functions (`rd.select`, `rd.when`, `rd.lit`, `rd.every`,
  `rd.generation`, `rd.element`, `rd.pi_signal`, `rd.adaptive_rate`,
  `rd.error_from`, `rd.is_converged`, `rd.stagnation`, `rd.is_stagnant`,
  `rd.p50/p95/p99`, `rd.quantile_stream`) are now classmethods on `Expr`
  (`rd.Expr.select(...)`, `rd.Expr.when(...)`, etc.), matching the library's
  `rd.Noun.factory` convention and keeping the `rd.*` namespace free of names
  that shadowed builtins. The redundant `rd.mean/min/max/stddev(metric)`
  shorthands were dropped — use `rd.Expr.select(metric).mean()`. Expressions
  remain experimental; no back-compat aliases are kept.

### Added

- **Expression DSL — `compile()` pass.** Constant subtrees fold to literals
  and affine chains (`a*x + b`, including the common
  `(x - target) / target * gain + 1` controller pattern) collapse to a single
  fused `Affine` node. Idempotent.
- **Expression DSL — new operators.** `stagnation` / `is_stagnant` for˚v
  patience-based plateau detection, `is_converged` for windowed convergence,
  `pi_signal` PI-control helper, and streaming P² quantiles (`p50`, `p95`,
  `p99`) with constant-memory online estimation.
- **New `neat-graph` example** demonstrating NEAT-style neuroevolution.
- **Workspace-wide `AnyValue` / `DataType` / `Field` re-exports** from
  `radiate-core` so downstream crates no longer need to depend on
  `radiate-utils` directly for these types.

### Changed

- **Recombination is substantially faster.** Survivor and offspring
  construction now share a single descending walk over the union of
  selected indices, emitting one `swap_remove` move plus `(total - 1)`
  clones per unique source rather than cloning every survivor and offspring
  independently. The species path applies the same optimization within
  each species' sub-population. In practice ~20–50% fewer `Phenotype`
  clones per generation.
- **Novelty search rewritten and expanded** (~+470 lines in
  `fitness/novelty.rs`) — broader archive management and behavior-distance
  options.
- **Engine builder restructured.** Survivor/offspring configuration is now
  factored into `builder/config.rs`, simplifying the top-level builder and
  making the per-step configuration explicit.
- **Metric subsystem cleanup.** Stale fixture data and brittle snapshot
  tests removed; `MetricView` is leaner and now covered by direct unit
  tests.
- **UI refactor.** Panel state moved off the `PanelId` dispatch model into
  a cleaner per-panel ownership scheme.
- **NSGA III** - The NSGA III has been greatly simplified and optimized with a better niching technique. This is a pretty nice improvement over the previous implementation.

### Fixed

- Various clippy and correctness cleanups across the alters, selectors,
  and engine crates (no behavior changes intended; flagged here only
  because several touched hot paths).
