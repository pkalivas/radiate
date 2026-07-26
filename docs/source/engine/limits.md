# Limits

___

A `Limit` is a condition checked once after every generation to decide whether the engine should keep going. Multiple limits can be active at once (via `Limit::Combined` in Rust, or by passing several to `.limit(...)`/`run(...)` in Python) — in every case, **the engine stops as soon as any one limit trips**.

| Limit | Stops when | Rust | Python |
|---|---|---|---|
| Generation | the generation counter reaches a target count | `Limit::Generation(n)` / `.until_generation(n)` | `rd.Limit.generations(n)` |
| Seconds | cumulative engine time reaches a duration | `Limit::Seconds(Duration)` / `.until_seconds(secs)` / `.until_duration(dur)` | `rd.Limit.seconds(secs)` |
| Score | the best score reaches or crosses a target (direction-aware per objective; for multi-objective, every component must pass) | `Limit::Score(Score)` / `.until_score(score)` | `rd.Limit.score(target)` |
| Convergence | improvement over a sliding window drops to (or below) an epsilon | `Limit::Convergence(window, epsilon, _)` / `.until_convergence(window, epsilon)` | `rd.Limit.convergence(window, threshold)` |
| Expr | a metric expression evaluates to `true` — see [Expressions](expressions.md#1-expression-limits) for the full recipe | `Limit::Expr(Expr)` / `.until_expr(expr)` | `rd.Limit.expr(expr)` |
| Combined | any one of a set of sub-limits trips | `Limit::Combined(vec![...])` / `.limit((a, b, c))` | `.limit(a, b, c)` |

!!! note "Metric-based limits"

    A predicate-based limit keyed to a named metric also exists (`Limit::Metric`/`until_metric` in Rust, `rd.Limit.metric(...)` in Python), but is intentionally left out of this page for now pending a closer look at its stop/continue polarity.

---

## Each limit on its own

=== ":fontawesome-brands-python: Python"

    ```python
    --8<-- "python/engine/limits.py:limits_each"
    ```

=== ":fontawesome-brands-rust: Rust"

    ```rust
    --8<-- "rust/engine/limits.rs:limits_score"
    ```

    ```rust
    --8<-- "rust/engine/limits.rs:limits_seconds"
    ```

    ```rust
    --8<-- "rust/engine/limits.rs:limits_convergence"
    ```

---

## Combining limits

Combining several at once — the engine stops on whichever trips first:

=== ":fontawesome-brands-python: Python"

    ```python
    --8<-- "python/engine/limits.py:limits_combined"
    ```

=== ":fontawesome-brands-rust: Rust"

    ```rust
    --8<-- "rust/engine/limits.rs:limits_combined"
    ```

!!! tip "Python requires at least one limit"

    Unlike Rust — where an unbounded `.iter()` or `run(closure)` is legal, since the closure or a `break` is itself the stop condition — Python's engine raises immediately if you call `.run()` or start iterating with no `Limit` attached anywhere. Always set at least one via `.limit(...)`.

---
