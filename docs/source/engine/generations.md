# Generations

___

Each time the engine advances, it produces a result describing that point in the evolutionary process — its generation number, the population, the best solution found, metrics, and more. Radiate has two types for this, and the difference between them is entirely about cost:

- **`Generation`** is an *owned* snapshot — safe to hold onto, pass around, or send across threads, at the cost of cloning the ecosystem to build it.
- **`GenerationView`** is a *borrowed* view over the exact same information — free to construct, but tied to the lifetime of the engine call that produced it.

!!! note "Terminology"

    `Epoch` is the associated type name on the `Engine` trait — it's generic, so any type implementing `Engine` could in principle use a different type for it. `Generation` is the concrete type `GeneticEngine` uses. `GenerationView` is the borrowed alternative described below. The docs use "epoch" informally to mean "whatever came back from advancing the engine," and `Generation` when the concrete type matters.

---

## Generation

This is the primary, owned type — the default epoch for the engine, and the only one exposed to Python. It contains:

- The generation number
- `Ecosystem` information (population, species, etc.)
- Score, which is the fitness of the best individual in the generation
- Value, which is the decoded value of the best individual
- Performance metrics (e.g., time taken)
- The Objective (max or min). The fitness objective being optimized, used for comparison and decision making during the evolutionary process.

### Single-Objective

=== ":fontawesome-brands-python: Python"

    ```python
    --8<-- "python/engine/generations.py:single_objective"
    ```

=== ":fontawesome-brands-rust: Rust"

    ```rust
    --8<-- "rust/engine/generations.rs:single_objective"
    ```

### Multi-Objective

When the engine is configured for multi-objective optimization, the `Generation` will have a `ParetoFront` attached to it. The only difference between the single-objective and multi-objective case is the availability of the `ParetoFront` and the shape of the `score` — a list of values, one per objective, instead of a single value.

=== ":fontawesome-brands-python: Python"

    ```python
    --8<-- "python/engine/generations.py:multi_objective"
    ```

=== ":fontawesome-brands-rust: Rust"

    ```rust
    --8<-- "rust/engine/generations.rs:multi_objective"
    ```

---

## GenerationView

!!! note "Rust only"

    `GenerationView` has no Python binding — Python always gets an owned `Generation` at the end of the engine run.

`GenerationView<'a, C, T>` borrows straight through to the engine's live context instead of cloning it — every accessor (`score()`, `value()`, `population()`, `metrics()`, …) mirrors `Generation`'s, just returning borrowed data instead of owned. Its only entry point is the argument to [`EngineRuntime::until(closure)`](runtime.md#combinators-actions), which is how Radiate lets you write an ad-hoc stopping condition without paying to construct a `Generation` on every generation just to check it:

```rust
--8<-- "rust/engine/generations.rs:generation_view"
```

If you find yourself wanting a `GenerationView` outside of `until(...)` — say, to inspect state mid-loop without the clone cost — that's a sign you may want the raw `Engine` trait's `step()`/`context()` directly instead (see [Runtime](runtime.md)).

---
