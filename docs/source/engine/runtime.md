# Engine Runtime

___

The engine runtime is a light wrapper around an engine which uses the engine's trait methods to drive. It runs _as much as possible_ in place, meaning it avoids unnecessary cloning, allocating, and recomputation. That being said, because its intent is primarily to provide the user with a dynamic and interactive way to run an engine, it will only run in place as long as the operations being called on it, allow it (more on this later). To actually execute the engine, you either iterate over it or use one of the convenience methods like `run()` or `last()` against attached [`Limit`s](limits.md). 

### Running

I mentioned above that the runtime tries to run the engine in place as much as possible, lets dive into what that really means. In technical terms, it means the runtime will attempt to reuse existing memory, computation, and references wherever it can by simply looping over the engine's operations using mutable state. The reason this boundary is drawn, is because a `Generation` is the main output of the engine, and the `Generation` contains data that, when cloned or modified frequently, can be expensive to maintain (quite expensive, in some cases). When the user specifies an operation(s) that require a fresh `Generation` or modify the engine's state in a way that prevents in-place execution, the runtime will fall back to creating new snapshots and performing the necessary recomputation. This 'swap' between the in-place and snapshot-based execution is handled automatically by the runtime and is transparent to the user. So, what would cause the runtime to fall back to snapshot-based execution? Any operation that needs to see a consistent view or an owned copy of the engine's state across multiple generations, or that modifies the state in a way that cannot be done in place, will trigger this behavior. Things like looping over the engine in a for loop, or calling `next(engine)` repeatedly, will force the runtime to create new `Generation` snapshots for each iteration. So, prefer using radiate's limit-driven methods whenever possible.

Common iterator methods like `take(n)` can be transformed into limit-driven operations under the hood, allowing in-place execution. This same sort of optimization is applied in many other iterator-like methods provided by the runtime, such as `until_generation(n)` or `until_seconds(t)`.

=== ":fontawesome-brands-python: Python"

    The engine is directly iterable (`for epoch in engine`), and `next(engine)` works too — both build a fresh `Generation` on every call.

    ```python
    --8<-- "python/engine/runtime.py:iterator_basic"
    ```

    Or drive it one epoch at a time:

    ```python
    --8<-- "python/engine/runtime.py:iterator_next"
    ```

    [`engine.run()`](#convenience-run) takes the `Limit`-driven path instead. Python requires at least one [`Limit`](limits.md) to be attached either way — there's no closure-based stop condition here.

=== ":fontawesome-brands-rust: Rust"

    `.iter()` hands back a runtime that implements the standard `Iterator` trait, so a `for` loop or an explicit `.next()` works directly:

    ```rust
    --8<-- "rust/engine/runtime.rs:iterator_basic"
    ```

    That same runtime also has its own `run()`/`.last()`, driven by attached [`Limit`s](limits.md) instead of by iterating. A closure passed straight to the engine's own `run(closure)`, without going through `.iter()` first, works too, but re-evaluates against a fresh `Generation` on every generation:

    ```rust
    --8<-- "rust/engine/runtime.rs:iterator_run"
    ```

    A closure over a borrowed [`GenerationView`](generations.md#generationview) via `.until(closure)` lets you write an ad-hoc stop condition without needing a full `Generation` — see [Generations](generations.md#generationview).

!!! warning "Always attach a stopping condition"

    The engine's iterator is a streaming, effectively infinite iterator — it produces epochs until a [limit](limits.md) trips, a `break`, or a `return`. Always attach one (or a method like `take`/`until`/`last` in Rust) unless you genuinely want to run indefinitely.

!!! note "Re-running the engine"

    **Rust**: a closure-based `run(closure)` call borrows the engine rather than consuming it, so calling it again on the same engine continues from wherever it left off. `.iter()`, by contrast, consumes the engine — once you've built a runtime from it, that engine value is gone.

    **Python**: `Engine` is a reusable *builder*, not a live engine. Every `for` loop, every `next()` call after `StopIteration`, and every `.run()` call constructs a brand-new engine from the builder's queued inputs. Calling `.run()` twice runs two independent evolutionary processes from a fresh population — it does not resume the first one.

---

## Combinators & Actions

Beyond the built-in stopping conditions, Radiate lets you compose several of them together and attach side effects like progress logging or periodic checkpointing.

=== ":fontawesome-brands-python: Python"

    Limits and side effects are both configured on the engine builder rather than chained onto an iterator — see [Limits](limits.md) for combining stop conditions, and [Convenience `run()`](#convenience-run) below for `log`/`checkpoint` options.

=== ":fontawesome-brands-rust: Rust"

    The runtime adds chainable methods for composing stop conditions and side effects: `until_score`, `until_generation`, `until_seconds`/`until_duration`, `until_convergence`, `until_expr`, `until(closure)`, and the generic `limit(...)` — see [Limits](limits.md) for what each one checks. `logging()`/`log_every(n)` print per-generation progress; `checkpoint(interval, path)`/`checkpoint_with(...)` persist engine state periodically.

    ```rust
    --8<-- "rust/engine/runtime.rs:iterator_actions"
    ```

---

### Convenience `run()`

For the common case — build an engine, run it to completion, get the final epoch — `run()` wraps the condition-driven loop without needing an explicit `for`/`while`:

=== ":fontawesome-brands-python: Python"

    ```python
    --8<-- "python/engine/runtime.py:iterator_convenience"
    ```

    `run()` also accepts `log`, `ui`, and `checkpoint` options:

    ```python
    --8<-- "python/engine/runtime_showcase.py:iterator_run_options"
    ```

=== ":fontawesome-brands-rust: Rust"

    ```rust
    --8<-- "rust/engine/runtime.rs:run_convenience"
    ```

---

## Control Interface

The engine provides a control interface for pausing, resuming, and stopping the evolutionary process from outside the run loop — for example, pausing or stepping through generations from another thread or in response to user input.

=== ":fontawesome-brands-python: Python"

    Not currently implemented.

=== ":fontawesome-brands-rust: Rust"

    ```rust
    --8<-- "rust/engine/runtime.rs:control"
    ```

---
