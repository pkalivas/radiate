# Genetic Programming

!!! warning ":construction: Under Construction :construction:"

    As of `11/13/2025`: These docs are a work in progress and may not be complete or fully accurate. Please check back later for updates.

___

Genetic Programming (GP) in Radiate enables the evolution of programs represented as **expression trees** and **computational graphs**. This powerful feature allows you to solve complex problems by evolving mathematical expressions, decision trees, neural network topologies, and more.

Radiate's GP implementation provides two core data structures: **Trees** for hierarchical expressions and **Graphs** for complex computational networks. Each offers unique capabilities for different problem domains. Both the `tree` and `graph` modules come with their own specific chromosomes, codecs and alters to evolve these structures effectively.

---

## Installation

To use Radiate's Genetic Programming features, you need to install the library with the appropriate feature flags.

=== ":fontawesome-brands-python: Python"

    ```bash
    pip install radiate
    ```

=== ":fontawesome-brands-rust: Rust"
    ```shell
    cargo add radiate -F gp

    # Or Cargo.toml
    [dependencies]
    radiate = { version = "x", features = ["gp", ...] }
    ```

---

## Arity

[Arity](https://en.wikipedia.org/wiki/Arity) is the cornerstone concept behind how `nodes` and `ops` function within Radiate's genetic programming system. Arity is a term used to describe the number of arguments or inputs a function takes. In the context of genetic programming, arity defines how many inputs a `node` or an `op` can accept in both `trees` and `graphs`. For `graphs` `arity` is used to determine how many incoming connections a `GraphNode` can have, while for `trees` it determines how many children a `TreeNode` can have. Radiate uses an enum to express `arity` defined in three variants:

1. **Zero**: The operation takes no inputs (e.g., constants).
2. **Exact(usize)**: The operation takes a specific number of inputs.
3. **Any**: The operation can take any number of inputs (e.g., functions like sum or product).

In most cases, the `tree` or `graph` will try it's best ensure that their node's `arity` is not violated, but it will ultimately be up to the user to ensure that the `arity` is correct.
