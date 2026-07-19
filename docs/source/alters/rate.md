# Rates

!!! warning "This page is a work in progress"

    The rate system spans the entire engine so the below documentation can jump around a bit (I'm working on this). These docs are still also being written (07/17/2026) so reader beware that some of the examples may be better understood after reading the whole documentation.

A `rate` controls how often an alterer fires. In Rust it's anything that implements `Into<Expr>`; in Python it's a `float` or an `Expr`. Either way, a bare number becomes a constant `Expr` automatically — `0.1` and `Expr.lit(0.1)` are equivalent. There's no separate "rate schedule" type: **`Expr`, the same metric-expression DSL described in [Expressions](../engine/expressions.md), is the only mechanism.** A fixed rate is just the simplest possible expression; a fully adaptive, metric-driven rate is built from the same combinators.

=== ":fontawesome-brands-python: Python"

    ```python
    --8<-- "python/alters/rate.py:applying"
    ```

=== ":fontawesome-brands-rust: Rust"

    ```rust
    --8<-- "rust/alters/rate.rs:applying"
    ```

## What actually happens each generation

Every `Mutate`/`Crossover` implementation exposes its rate as a `RateSet` wrapping at least one `Expr`. Once per generation, before any genes are touched, the engine evaluates that `Expr` against the live `MetricSet` and caches the results — so no matter how many genes or pairs the operator visits that generation, the expression(s) run exactly once (again, once per generation). The cached value(s) are then used as a Bernoulli probability (in most cases): a mutator flips a weighted coin per gene, a crossover flips one per parent pair.

This means a rate expression can read *anything* the engine has already computed that generation — the rolling mean of the best score, how many species exist, how long the population has gone without improving — and the operator's behavior shifts accordingly, with zero coordination code on your part (wooo!!).

**The same mechanism drives more than alterer rates.** The species distance threshold (see [Adaptive thresholds](../diversity/species.md#adaptive-thresholds)) and `target_species_count`'s internal PID-style controller are both ordinary `Expr`s evaluated through the identical `RateSet` path. If you understand rates, you already understand how the engine's other live-tunable knobs work.

---

Use the table below to pick a starting recipe, then see its section for the full expression. All of these are just `Expr`s — mix, extend, or replace any of them. These may or may not suit your needs, so use them sparingly and more so as a starting point than a final solution. The engine is designed to let you build your own expressions from scratch, and the combinators are composable enough that you can express almost anything you want.

**These are just some examples of common shapes that _might_ be useful.**

| Recipe | Behavior | Reach for it when… |
|---|---|---|
| [Constant](#constant) | one fixed value | the default — you don't need it to change |
| [Linear ramp](#linear-ramp) | smooth transition from a start value to an end value | you want to shift exploration ↔ exploitation gradually over a known window |
| [Stepped](#stepped) | jumps to a new value at chosen generations | your run has distinct, known phases |
| [Periodic](#periodic) | alternates between two values on a fixed cadence | you want to periodically re-inject exploration to escape local optima |
| [Exponential decay](#exponential-decay) | fast initial change that levels off | you want a high starting rate that decays quickly and then holds |
| [Metric-driven](#metric-driven) | reacts to live population state, not just generation count | you want the rate to respond to what's actually happening in the run |

## Constant

The default. No `Expr` construction needed — pass the number directly.

=== ":fontawesome-brands-python: Python"

    ```python
    --8<-- "python/alters/rate.py:constant"
    ```

=== ":fontawesome-brands-rust: Rust"

    ```rust
    --8<-- "rust/alters/rate.rs:constant"
    ```

## Linear ramp

`generation index / duration`, clamped to `[0, 1]`, then scaled between a start and end value. Read `metric_names::INDEX` (Rust) / `Expr.generation()` (Python) as "the generation counter, exposed as just another metric."

=== ":fontawesome-brands-python: Python"

    ```python
    --8<-- "python/alters/rate.py:linear_ramp"
    ```

=== ":fontawesome-brands-rust: Rust"

    ```rust
    --8<-- "rust/alters/rate.rs:linear_ramp"
    ```

## Stepped

Nested `when / then / otherwise` on the generation index. Each branch is itself an `Expr`, so conditionals compose arbitrarily deep.

=== ":fontawesome-brands-python: Python"

    ```python
    --8<-- "python/alters/rate.py:stepped"
    ```

=== ":fontawesome-brands-rust: Rust"

    ```rust
    --8<-- "rust/alters/rate.rs:stepped"
    ```

## Periodic

`every(n)` is stateful and ignores its input entirely — it just fires `true` once every `n` evaluations. Paired with `then/otherwise`, it swaps between two values on a fixed cadence.

=== ":fontawesome-brands-python: Python"

    ```python
    --8<-- "python/alters/rate.py:periodic"
    ```

=== ":fontawesome-brands-rust: Rust"

    ```rust
    --8<-- "rust/alters/rate.rs:periodic"
    ```

## Exponential decay

There's no built-in "exponential" node — it's a half-life formula built from `pow`: `end + (start - end) * 0.5^(index / half_life)`.

=== ":fontawesome-brands-python: Python"

    ```python
    --8<-- "python/alters/rate.py:exponential_decay"
    ```

=== ":fontawesome-brands-rust: Rust"

    ```rust
    --8<-- "rust/alters/rate.rs:exponential_decay"
    ```

## Metric-driven

This is the recipe none of the others can express: a rate that responds to the population's *actual behavior* instead of a schedule fixed in advance. Two common shapes —

- **Gated on stagnation.** Boost mutation once the best score has gone `patience` generations without improving by more than `epsilon`. Python's `Expr.is_stagnant(...)` is sugar for `Expr.select(metric).stagnation(epsilon).gte(patience)`, shown explicitly on the Rust side.
- **Tracking a continuous signal.** `score.volatility` (coefficient of variation of the best score) is a good general-purpose "how settled is the population" signal — dial the rate down as it drops.

=== ":fontawesome-brands-python: Python"

    ```python
    --8<-- "python/alters/rate.py:metric_driven"
    ```

=== ":fontawesome-brands-rust: Rust"

    ```rust
    --8<-- "rust/alters/rate.rs:metric_driven"
    ```

For the full combinator reference — aggregations, comparisons, arithmetic, querying metrics, derived metrics, expression limits — see [Expressions](../engine/expressions.md). Everything there applies equally to rates; there's nothing rate-specific about the DSL itself.

!!! tip "Rates aren't clamped for you"

    The engine uses whatever the expression evaluates to as a Bernoulli probability. Nothing forces that value into `[0, 1]` automatically — if you're deriving a rate from an unbounded signal (a rolling stddev, a slope, an error term), end the expression with `.clamp(min, max)` as in the examples above.
