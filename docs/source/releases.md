# Release Notes

## v1.3.1 - py 0.0.15
- 2026-09-13
- [Release](https://github.com/pkalivas/radiate/releases/tag/v1.3.1)

`Rate` is fully replaced by the expression DSL, events/checkpointing/stopping move onto the engine builder, and the Python operator API is reorganized into namespaces (`Select.*`/`Cross.*`/`Mutate.*`/`Dist.*`/`Limit.*`/`Filter.*`/`Fitness.*`). Also new: a population-filter stage for stagnation recovery, adaptive species-count targeting, a `BitFlipMutator`, f64 support for GP graphs/trees, and three new TUI dashboard tabs. Pareto front calculation's should be _much_ faster now; buffers are cached & reused whenever possible, efficent sorting, and in-place crowding distance calculation.

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

## v1.3.0 - py 0.0.14
- 2026-06-20
- [Release](https://github.com/pkalivas/radiate/releases/tag/v1.3.0)
  
Radiate has reached `1.3.0`! This release includes a major refactor of the engine's iteration model, a new expression DSL pass and operators, a substantial NSGA III simplification and optimization, a NEAT implementation refactor, and various other improvements and cleanups across the codebase. The `engine` is now _much_ more efficient, and the new expression DSL features should make it easier to implement complex adaptive behaviors without custom code. The user guide has been rebuilt around testable snippets, and a new guide on diversity and speciation has been added. 

Check the [changelog](https://github.com/pkalivas/radiate/blob/master/CHANGELOG.md) for a full list of changes.

## v1.2.22 - py 0.0.13

- 2026-04-25
- [Release](https://github.com/pkalivas/radiate/releases/tag/v1.2.22)

### Breaking changes:

* Changed the checkpointing feature in python to use `.pkl` as a default extension instead of `.json`. It fits better within the python ecosystem. 
* Changed the metric names to use `.` as a separator instead of `_`. This allows for better organization and grouping of metrics. For example, `scores.best` instead of `best_scores`.

### Other

Speed improvements centered around engine steps. 

### Additions

Added a new crate `radiate-expr` which includes expressions (think polars) to extend the metric and rating systems. This greatly improves the flexibility of dynamic rates (mutation/crossover/species thresholds) and allows users to define their own rating systems. Along with the rate improvements, this extends into the engine itself by allowing users to define their own metrics and use them in the aforementioned dynamic rates - or simply just to track the engine. 

Refactored `radiate-ui` to give much more insight into the engine and the metrics it produces. Included a new search bar and species panel to quickly find and visualize specific species and their members.

In python, radiate now supports optional features (check the user guide installation section for specific info). This lets users opt in to specific integrations within the python ecosystem (e.g. pandas, polars, matplotlib, torch, numpy) without needing to install a bunch of dependencies they may not need.

For checkpointing, new traits were added: `CheckpointWriter` & `CheckpointReader` to let users define their own ways of saving checkpoints.

**[Full Changelog](https://github.com/pkalivas/radiate/compare/v1.2.21...v1.2.22)**

---

## v1.2.21 - py 0.0.11

- 2026-02-22
- [Release](https://github.com/pkalivas/radiate/releases/tag/v1.2.21)


### Breaking changes:

Changed the `FloatGene<T>` to take a generic <F> parameter to take either an `f32` or `f64` value. 

* The above is a breaking change. Just add the generic type to your `FloatGene`, `FloatChromosome`, or `FloatCodec` if needed.

Massive expansion of python's api - check the [docs](https://pkalivas.github.io/radiate/) for usage. We moved towards a builder pattern for the engine and added better type hinting.

**[Full Changelog](https://github.com/pkalivas/radiate/compare/v1.2.20...v1.2.21)**

---

## v1.2.20 - py 0.0.10

- 2025-12-15
- [Release](https://github.com/pkalivas/radiate/releases/tag/v1.2.20)

Adding radiate-ui crate, bug fixes, & speed improvements. 

I split up some functionality into a new crate radiate-utils and have added a new feature radiate-ui for a tui user interface through [ratitui](https://ratatui.rs). Some small bug fixes, code simplifications, and some nice little speed improvements. 

**[Full Changelog](https://github.com/pkalivas/radiate/compare/v1.2.19...v1.2.20)**

---

## v1.2.19 - py 0.0.9

- 2025-11-11
- [Release](https://github.com/pkalivas/radiate/releases/tag/v1.2.19)

Adding support for experimental [PGM](https://en.wikipedia.org/wiki/Graphical_model) or Probabilistic graphical models through the GP feature (crate).

!!! note

    PGM support was experimental and has since been removed; it is no longer part of Radiate.

Major cleanup or unused code and massive graph performance improvements through the use of [smallvec](https://docs.rs/smallvec/latest/smallvec/) as connections instead of BTreeSets. 

Improving eventing system through cleaner code and removing redundant events.

Introducing radiate-error (RadiateError) into the core crates ad requiring its usage in certain traits (Problem mainly). We also use this error type in py-radiate and allow it to bubble up into python's type system too. 

Brining metrics to the forefront in python. 

**[Full Changelog](https://github.com/pkalivas/radiate/compare/v1.2.18...v1.2.19)**

---

## v1.2.18 - py 0.0.8

- 2025-09-27
- [Release](https://github.com/pkalivas/radiate/releases/tag/v1.2.18)

Fixing subtle bug in recurrent graphs where a random seed wasn't being respected, leading to non-deterministic behavior in some cases. This fix ensures that all random operations within recurrent graphs are consistent and reproducible when a seed is provided. 

Added three new types of graphs:

- **LSTM** (Long Short-Term Memory) Graphs: These are a type of recurrent neural network (RNN) that can learn long-term dependencies.
- **GRU** (Gated Recurrent Unit) Graphs: Similar to LSTMs, GRUs are a type of RNN that are simpler and often more efficient.
- **Mesh** Graphs: Graphs structured in a mesh topology.

---

## v1.2.17 - py 0.0.7

- 2025-09-04
- [Release](https://github.com/pkalivas/radiate/releases/tag/v1.2.17)

In response to github issue [#23](https://github.com/pkalivas/radiate/issues/23).

Ensuring that FloatGenes/IntGene<T>'s respect their bounds during mutation and crossover. This was a bug where mutated or crossovered genes could exceed their defined bounds, which could lead to invalid individuals in the population. This fix ensures that all FloatGenes/IntGene<T>'s remain within their specified bounds after any genetic operation. Also some optimizations and code cleanup for py-radiate. Large additions to tests.

Also adding new mutator: `JitterMutator` for FloatGenes. This mutator adds a small random value (jitter) to each gene, controlled by a `magnitude` parameter.

---

## v1.2.16

- 2025-08-19
- [Release](https://github.com/pkalivas/radiate/releases/tag/v1.2.16)

In response to github issue [#22](https://github.com/pkalivas/radiate/issues/22).

Adding support for batch fitness functions and batch engine problems through a new trait (BatchFitnessFn). Some small cleanup on other fitness functions and some chromosome operators.

---

## v1.2.15 - py 0.0.6

- 2025-08-10
- [Release](https://github.com/pkalivas/radiate/releases/tag/v1.2.15)

Adding Novelty Search to python and refactoring engine building across the rust/python bridge. Improving python's speed. Adding type checking to python and upgrading python package to >= python 3.12 to support new python generics. Improving docs to reference new functionality.

New alters:

  * EdgeRecombinationCrossover for PermutationGenes 
  * PolynomialMutator for chromosomes with FloatGenes

Added code path in alters for dynamic mutation/crossover rates. This is in early dev, but an be seen in PolynomialMutator.

---

## v1.2.14 - py 0.0.4

-  2025-07-05
-  [Release](https://github.com/pkalivas/radiate/releases/tag/v1.2.14)

Added support for novelty search, fitness-based novelty, and combined novelty and fitness search. Improved documentation and examples. Improved traits for `Engine` and introduced one for `FitnessFn`. Bug fixes for pareto fronts and engine iterators.