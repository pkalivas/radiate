
# Regression

In machine learning it's common to have a regression task. This is where you have a set of inputs and outputs, and you want to find a function that maps the inputs to the outputs. In Radiate, we can use genetic programming to evolve a `tree` or `graph` to do just that. The regression `problem` is a special type of `problem` that simplifies this process. It provides functionality to normalize/standardize/OHE the inputs and outputs, as well as calculate the fitness of a `genotype` based on how well it maps the inputs to the outputs.

The regression problem (fitness function) takes a set of inputs and outputs, and optionally a loss function. Each loss reduces the per-sample error between the genotype's predictions and the targets to a single score (averaged over the dataset), which the engine then minimizes:

- **MSE** (mean squared error) — the default; averages the *squared* error, so large misses are penalized heavily. A solid general-purpose choice.
- **MAE** (mean absolute error) — averages the *absolute* error; less sensitive to outliers than MSE.
- **CrossEntropy** (`XEnt`) — averages `-target · ln(pred)` (predictions clamped just above 0); use it for probability / classification-style outputs.
- **Diff** — averages the raw *signed* difference (`target - pred`). The simplest option, but because over- and under-predictions cancel out it's rarely a good standalone target — handy for custom setups.

Let's take a quick look at how we would put together a regression problem using a `graph`. A `tree` works exactly the same way — just swap the `GraphCodec` (or `rd.Engine.graph(...)`) for its tree equivalent.

=== ":fontawesome-brands-python: Python"

    !!! tip "GP regression and the GIL"
    
        The regression fitness function runs purely in rust, as such any executor can be used (including multi-threaded executors) without any issues with the GIL regardless of the python version being used.

    ```python
    --8<-- "python/gp/regression.py:graph_regression"
    ```

    Radiate is also fully compatible with DataFrame libraries like Pandas and Polars, so wiring DataFrames into regression problems is straightforward. For an example we'll use [polars](https://pola.rs) as it's quickly becoming the go-to DataFrame library in python. Using the `.regression(..)` method below is an attempt to simplify the configuration of regression problems. It also automatically switches the engine's optimization target to minimization as most regression losses are minimized.

    The call to this method is flexible and allows you to specify the target columns, feature columns, and loss function. But it also handles the simple case we saw above where we just want to provide a list of inputs and outputs.

    ```python
    --8<-- "python/gp/regression.py:dataframe_regression"
    ```

=== ":fontawesome-brands-rust: Rust"

    ```rust
    --8<-- "rust/gp/regression.rs:graph_regression"
    ```


---

## Datasets

`radiate` ships a set of small, well-known datasets so you can try out a regression problem without bringing your own data. Datasets that are generated randomly follow the global [random seed](../misc/random.md), so they are reproducible.

| Dataset | Kind | Shape |
|---|---|---|
| `iris` | classification | 150 samples, 4 features, 3 one-hot labels (shuffled) |
| `xor` | classification | the 4-row XOR truth table |
| `parity(bits)` | classification | every `bits`-wide bit pattern, labelled 1 when an odd number of bits are set |
| `moons(n, noise)` | classification | two interleaving half circles, 2 features, a 0/1 label |
| `circles(n, noise)` | classification | two concentric circles, 2 features, a 0/1 label |
| `spirals(n, noise)` | classification | two intertwined spirals, 2 features, a 0/1 label |
| `simple_memory(n)` | sequence | `n` steps of a repeating 4-step pattern (input pulses every 4th step, the target the step before), for testing recurrent graphs |
| `cubic` | regression | `4x³ - 3x² + x` at 20 evenly spaced points |
| `koza1` | regression | `x⁴ + x³ + x² + x` at 20 random points in [-1, 1) |
| `nguyen7` | regression | `ln(x + 1) + ln(x² + 1)` at 20 random points in [0, 2) |
| `pagie1` | regression | `1/(1 + x⁻⁴) + 1/(1 + y⁻⁴)` on a 26 × 26 grid over [-5, 5] |

A dataset can be shuffled, split into two parts, and have its feature columns standardized (mean 0, standard deviation 1) or normalized (scaled to [0, 1]). The two-class datasets (`moons`, `circles`, `spirals`) come out grouped by class, so shuffle them before splitting.

=== ":fontawesome-brands-python: Python"

    `rd.DataSet` exposes its features and labels as numpy arrays (`.features`, `.labels`, or both with `.pair()`), which plug straight into `.regression(...)`. Each builder takes an optional `dtype`.

    ```python
    --8<-- "python/gp/regression.py:datasets"
    ```

=== ":fontawesome-brands-rust: Rust"

    `DataSet<F>` is generic over `f32`/`f64` and can be passed directly to `Regression::new`. `friedman1(n, noise)` (5 informative features plus 5 noise features) is also available.

    ```rust
    --8<-- "rust/gp/regression.rs:datasets"
    ```

---

More robust examples can be found in the next section or in the [tree](https://github.com/pkalivas/radiate/tree/master/examples/rust/trees) and [graph](https://github.com/pkalivas/radiate/tree/master/examples/rust/graphs) examples in the git repository.

---
