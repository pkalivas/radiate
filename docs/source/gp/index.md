# Genetic Programming

___

Genetic Programming (GP) in Radiate enables the evolution of programs represented as **expression trees** and **computational graphs**. This powerful feature allows you to solve complex problems by evolving mathematical expressions, decision trees, neural network topologies, and more.

Radiate's GP implementation provides two core data structures: **Trees** for hierarchical expressions and **Graphs** for complex computational networks. Each offers unique capabilities for different problem domains. Both the `tree` and `graph` modules come with their own specific chromosomes, codecs and alters to evolve these structures effectively.

---

## Trees vs Graphs

Both structures evolve programs, but they suit different problems:

| | Tree | Graph |
|---|---|---|
| Shape | rooted, acyclic hierarchy | directed network; connections can form cycles |
| Evolves | the expression's shape and its ops | topology, connectivity, and edge weights |
| Reach for it when | symbolic regression, math expressions, decision logic | neuroevolution / neural-network topologies (NEAT-style), including recurrent models |

If you're fitting a formula to data, start with a [Tree](trees.md). If you're evolving a network architecture, reach for a [Graph](graph.md).

This section is organized as:

| Page | Covers |
|---|---|
| [Ops](op.md) | the operation catalog — functions, variables, constants — and writing custom ops |
| [Node](node.md) | the roles a node can play in a tree or graph, and the `NodeStore` |
| [Tree](trees.md) | tree structure, its codec, and tree-specific alterers |
| [Graph](graph.md) | graph structure, architectures (directed / weighted / recurrent / LSTM / GRU), codec, and alterers |
| [Regression](regression.md) | evolving a tree or graph to fit input/output data |

---

## Installation

In Rust, GP lives behind the `gp` feature flag, so enable it in your `Cargo.toml`. The Python package ships with GP included — a plain `pip install radiate` is all you need.

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

1. **Zero**: the operation takes no inputs. Constants and input variables are `Zero`.
2. **Exact(n)**: the operation takes exactly `n` inputs. Most math ops are fixed-arity — `add`, `sub`, and `mul` are `Exact(2)`, `sin` is `Exact(1)`.
3. **Any**: the operation accepts any number of inputs. Variadic ops like `sum`, `prod`, `min`, and `max` are `Any`.

In most cases, the `tree` or `graph` will try its best to ensure that their node's `arity` is not violated, but it will ultimately be up to the user to ensure that the `arity` is correct.
