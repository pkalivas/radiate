# Expressions

Radiate includes a composable expression system that lets you query and transform the engine's metric state at runtime. Expressions are stateful, lazily-evaluated trees — each call to `.apply()` or an internal engine dispatch consumes one "tick" of any stateful nodes (rolling windows, schedules, etc.). This system was designed to be extremely similar to [polars' expression API](https://pola-rs.github.io/polars/py-polars/html/reference/expressions/index.html) to leverage the same mental model of lazy evaluation and chaining, but adapted to radiate's needs.

Expressions are used in three places within the engine:

1. **Termination conditions** — stop evolution when an expression evaluates to `true`
2. **Derived metrics** — register expressions that run every generation and write their output back into the `MetricSet`
3. **Dynamic rates** — drive alterer rates from metric values rather than a fixed schedule

---

## Building Expressions

=== ":fontawesome-brands-python: Python"

    The expression DSL is available directly from the `radiate` package:

    ```python
    --8<-- "python/engine/expressions.py:building"
    ```

=== ":fontawesome-brands-rust: Rust"

    The `expr` module provides the building-block constructors:

    ```rust
    --8<-- "rust/engine/expressions.rs:building"
    ```

---

## Aggregations

Expressions can aggregate over accumulated history using a rolling window or directly over a collection.

=== ":fontawesome-brands-python: Python"

    ```python
    --8<-- "python/engine/expressions.py:aggregations"
    ```

=== ":fontawesome-brands-rust: Rust"

    ```rust
    --8<-- "rust/engine/expressions.rs:aggregations"
    ```

---

## Comparisons and Logic

Expressions support standard comparison and boolean operators. These always produce a `Bool` result.

=== ":fontawesome-brands-python: Python"

    Python operator overloads are supported, so you can write expressions naturally:

    ```python
    --8<-- "python/engine/expressions.py:comparisons"
    ```

=== ":fontawesome-brands-rust: Rust"

    ```rust
    --8<-- "rust/engine/expressions.rs:comparisons"
    ```

---

## Arithmetic

=== ":fontawesome-brands-python: Python"

    Python arithmetic operators are supported:

    ```python
    --8<-- "python/engine/expressions.py:arithmetic"
    ```

=== ":fontawesome-brands-rust: Rust"

    ```rust
    --8<-- "rust/engine/expressions.rs:arithmetic"
    ```

---

## Conditional Expressions

`when / then / otherwise` composes a ternary branch. Both branches must be `Expr` values.

=== ":fontawesome-brands-python: Python"

    ```python
    --8<-- "python/engine/expressions.py:conditional"
    ```

=== ":fontawesome-brands-rust: Rust"

    ```rust
    --8<-- "rust/engine/expressions.rs:conditional"
    ```

---

## Schedule: `every`

`every(n)` fires `true` once every `n` calls and `false` otherwise. It is purely stateful — it ignores its input entirely. Combined with `when / then / otherwise`, it produces an expression that returns different values at different cadences.

=== ":fontawesome-brands-python: Python"

    ```python
    --8<-- "python/engine/expressions.py:schedule"
    ```

=== ":fontawesome-brands-rust: Rust"

    ```rust
    --8<-- "rust/engine/expressions.rs:schedule"
    ```

---

## Querying Metrics

Expressions are evaluated against the engine's `MetricSet`. Meaning any metric inside the metric set can be selected and transformed with expressions. The most commonly used metrics are documented in the [Metrics reference](../engine/metrics.md), but you can also select any custom metric you've registered via `register_metrics` or that the engine produces internally.

Additional metrics are available when the engine is configured for [species-based diversity](../diversity/index.md) (`species.count`, `species.age`, etc.) or [multi-objective optimization](../objectives.md) (`front.size`, `front.entropy`, etc.).

By default `expr::select("metric_name")` reads `last_value`. To interpret the value as a duration, chain `.time()` before the aggregation:

=== ":fontawesome-brands-python: Python"

    ```python
    --8<-- "python/engine/expressions.py:querying"
    ```

=== ":fontawesome-brands-rust: Rust"

    ```rust
    --8<-- "rust/engine/expressions.rs:querying"
    ```

---

## Integration:

### 1: Expression Limits

An `Expr` that returns a `bool` can be used as a termination condition. The engine stops as soon as the expression evaluates to `true`.

=== ":fontawesome-brands-python: Python"

    ```python
    --8<-- "python/engine/expressions.py:limit_expr"
    ```

    You can also combine an expression limit with other limits:

    ```python
    --8<-- "python/engine/expressions.py:limit_combined"
    ```

=== ":fontawesome-brands-rust: Rust"

    ```rust
    --8<-- "rust/engine/expressions.rs:limit_expr"
    ```

---

### 2: Derived Metrics

You can register named expressions that are evaluated against the `MetricSet` at the end of every generation. Their output is written back as new metrics (tagged `Expr`), making them available to downstream expressions, limits, events, and the dashboard. This means you can create your own metrics!

=== ":fontawesome-brands-python: Python"

    Pass kwargs to `.metrics()` on the engine builder:

    ```python
    --8<-- "python/engine/expressions.py:derived_metrics"
    ```

    Derived metrics can also reference each other, as long as the referenced metric was registered first. They can also be used directly in a `Limit.expr`:

    ```python
    --8<-- "python/engine/expressions.py:derived_metrics_limit"
    ```

=== ":fontawesome-brands-rust: Rust"

    Pass a `Vec<impl Into<NamedExpr>>` to `.register_metrics()` on the builder:

    ```rust
    --8<-- "rust/engine/expressions.rs:derived_metrics"
    ```

    Derived metrics can also feed expression limits:

    ```rust
    --8<-- "rust/engine/expressions.rs:derived_metrics_limit"
    ```

---

### 3: Dynamic Rates

An expression can also drive an alterer's rate, a species threshold, or any other `rate`-typed parameter. The expression is evaluated against the `MetricSet` each generation, and the result is used as the rate for that step. See [Rates](../alters/rate.md) for the full set of recipes.

=== ":fontawesome-brands-python: Python"

    Pass the `Expr` directly wherever a rate is accepted:

    ```python
    --8<-- "python/engine/expressions.py:dynamic_rates"
    ```

=== ":fontawesome-brands-rust: Rust"

    Pass the `Expr` directly wherever a rate is accepted:

    ```rust
    --8<-- "rust/engine/expressions.rs:dynamic_rates"
    ```

---

## Example

So, what does all this do in practice? Well, let's say you opt-in to using speciation and as you test, you discover that ideally, your problem gets solved best with ~4 species. Well, radiate doesn't offer a 'target species' option out of the box, but using expressions you can build a dynamic rate (threshold in this case) that encourages the engine to maintain that number of species. Below we build a distance metric that acts as a feedback loop which combines several species-level metrics, then use it as the distance function for speciation. We also register two derived metrics to track the average distance and species count over time. 

=== ":fontawesome-brands-python: Python"

    Then, using radiate's built-in `MetricCollector` subscriber, we plot those metrics over time to see how our registered distance signal correlates with species count and overall diversity.

    ```python
    --8<-- "python/engine/expressions_showcase.py:example"
    ```

=== ":fontawesome-brands-rust: Rust"

    ```rust
    --8<-- "rust/engine/expressions.rs:example"
    ```

The above engine will produce something (this was produced in python) like the following. We can see that the rolling 10 generation species threshold (orange) is adjusting dynamically according to the species count, resulting in an average of around 4 species (green) and a corresponding species count (blue).

<figure markdown="span">
    ![expr_spec_threshold](../../assets/rates/expr_spec_threshold.png){ width="600" }
</figure>

This same sort of logic (combining multiple signals into a single metric/rate/signal) can be applied to other aspects of the engine as well, like alterer rates. For example, you might want to start with a high mutation rate to encourage exploration, but then dial it back as the population converges. By building a dynamic rate that combines several metrics of convergence (e.g., score volatility, species count, etc.) you can get a more robust signal for when to dial back mutation than just using generation count or best score alone.

---

## Tips

- **Expressions are stateful.** Rolling windows and `every` counters accumulate state across generations. If you reuse the same `Expr` object in two places (e.g., both a derived metric and a limit), they share state — construct separate instances instead.
- **Derived metrics update before limits are checked.** This means a `Limit::Expr` can reference a metric registered via `register_metrics` and see that generation's fresh value.
- **Expressions that select unknown metric names return `Null`.** Arithmetic on `Null` propagates `Null`; comparisons against `Null` return `false`.
- **`every(n)` counts calls, not generations.** If the same expression is called multiple times per generation, the counter advances faster than you might expect. Registered derived metrics are called exactly once per generation.
