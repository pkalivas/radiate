# Example

___

A handful of small, focused examples rather than one big walkthrough — each one contrasts a different way of driving the engine. The last one is the important one: it's the case where `run()`/`last()` genuinely isn't enough and you need the iterator instead.

---

## A single limit

The simplest shape: build, attach one limit, `run()`.

=== ":fontawesome-brands-python: Python"

    ```python
    --8<-- "python/engine/example.py:single_limit"
    ```

=== ":fontawesome-brands-rust: Rust"

    ```rust
    --8<-- "rust/engine/example.rs:single_limit"
    ```

---

## Combined limits

Attach several limits — the engine stops on whichever trips first, so this run stops well before 10,000 generations if the score target is hit sooner.

=== ":fontawesome-brands-python: Python"

    ```python
    --8<-- "python/engine/example.py:combined_limits"
    ```

=== ":fontawesome-brands-rust: Rust"

    ```rust
    --8<-- "rust/engine/example.rs:combined_limits"
    ```

---

## An ad-hoc limit

Rust only: when none of the built-in `Limit` variants fit, `until(closure)` takes an arbitrary predicate over a [`GenerationView`](generations.md#generationview) — still routed through `run()`/`last()` under the hood, so no `Generation` gets built until the predicate finally returns `true`.

```rust
--8<-- "rust/engine/example.rs:until_closure"
```

---

## When you actually need the iterator

Limits only answer "should I stop?" — they can't hand you the intermediate state itself. If you need to *act* on every generation as it happens (stream scores somewhere, update a live plot, react to a pause request), a `Limit` can't do that job no matter how it's composed — you need the real per-generation `Generation`, which means the iterator, not `run()`.

=== ":fontawesome-brands-python: Python"

    ```python
    --8<-- "python/engine/example.py:iterator_fallback"
    ```

=== ":fontawesome-brands-rust: Rust"

    ```rust
    --8<-- "rust/engine/example.rs:iterator_fallback"
    ```

---
