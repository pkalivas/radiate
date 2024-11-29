<!-- <h1 text-align="center">Radiate</h1> -->
<center>
  <h1>Radiate</h1>
  <img src="/docs/assets/radiate.png" height="100">

![master branch checks][master_branch_checks] ![Crates.io][crates_link] ![Crates.io License][license] ![Static badge][static_evolution_badge]
</center>

[crates_link]: https://img.shields.io/crates/v/radiate

[master_branch_checks]: https://img.shields.io/github/check-runs/pkalivas/radiate/master

[license]: https://img.shields.io/crates/l/radiate

[static_evolution_badge]: https://img.shields.io/badge/evolution-genetics-default

[rust_badge]: https://img.shields.io/badge/rust-%23000000.svg?logo=rust&logoColor=orange

[jenetics_link]: https://github.com/jenetics/jenetics

[genevo_link]: https://github.com/innoave/genevo

[radiate_legacy]: https://github.com/pkalivas/radiate.legacy


For more details check the [Documentation](https://pkalivas.github.io/radiate/)

Radiate is a powerful Rust library designed for implementing genetic algorithms and artificial evolution techniques. It
provides a flexible framework for creating, evolving, and optimizing solutions to complex problems using principles
inspired by natural selection and genetics. This library is suitable for researchers, developers, and enthusiasts
interested in evolutionary computation and optimization.

---

Large insperation for this library coming from other genetic algorithm libraries:
[Jenetics][jenetics_link]: A Java implementatino of GAs.
[genevo][genevo_link]: Popular rust GA.
[radiate_legacy][radiate_legacy]: Previous implemenation of this library with direct encoding.

## Usage

Add to cargo.toml

```toml
[dependencies]
radiate = "1.2.3"
```


<!-- ## Examples

The radiate-examples directory contains several examples demonstrating the capabilities of the library, including:

* **[Min-Sum](https://github.com/pkalivas/radiate/blob/master/radiate-examples/min-sum/src/main.rs)**: An example of
  minimizing a sum of integers.
* **[N-Queens](https://github.com/pkalivas/radiate/blob/master/radiate-examples/nqueens/src/main.rs)**: A classic
  problem in which the goal is to place N queens on a chessboard such that no two queens threaten each other.
* **[Knapsack](https://github.com/pkalivas/radiate/blob/master/radiate-examples/knapsack/src/main.rs)**: Another classic
  problem for evolutionary algorithms.
* **[Regression Graph](https://github.com/pkalivas/radiate/blob/master/radiate-examples/regression-graph/src/main.rs)**:
  Evolve a ```Graph<f32>``` (essentially a graph based neural network) for regression analysis.
*
    *

*[Simple Memory Graph](https://github.com/pkalivas/radiate/blob/master/radiate-examples/simple-memory-graph/src/main.rs)
**: Evolve a ```Graph<f32>``` (Neural Network) using recurrent connections for Neural Network based memory.

* **[XOR Graph](https://github.com/pkalivas/radiate/blob/master/radiate-examples/xor-graph/src/main.rs)**: Evolve a
  ```Graph<f32>``` to solve the classic [XOR problem](https://dev.to/jbahire/demystifying-the-xor-problem-1blk). -->