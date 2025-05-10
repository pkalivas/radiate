<h1 align="center">Radiate</h1>
<p align="center">
  <img src="/docs/assets/radiate.png" height="100">
</p>

<span align="center">

  ![master branch checks][master_branch_checks] ![Crates.io][crates_link] ![pypi.org][pypi_badge] ![License][license] ![Static badge][static_evolution_badge]

</span>

[crates_link]: https://img.shields.io/crates/v/radiate
[master_branch_checks]: https://img.shields.io/github/check-runs/pkalivas/radiate/master
[license]: https://img.shields.io/crates/l/radiate
[static_evolution_badge]: https://img.shields.io/badge/evolution-genetics-default
[pypi_badge]: https://img.shields.io/pypi/v/radiate?color=blue
[rust_badge]: https://img.shields.io/badge/rust-%23000000.svg?logo=rust&logoColor=orange
[jenetics_link]: https://github.com/jenetics/jenetics
[genevo_link]: https://github.com/innoave/genevo
[radiate_legacy]: https://github.com/pkalivas/radiate.legacy
 
For more details check radiate's [website](https://pkalivas.github.io/radiate/) or cargo [docs](https://docs.rs/radiate/latest/radiate/).

As of `05/09/2025` Python bindings are in active development.

Radiate is a powerful Rust library designed for implementing genetic algorithms and artificial evolution techniques. It
provides a fast and flexible framework for creating, evolving, and optimizing solutions to complex problems using principles
inspired by natural selection and genetics. With an intuitive, 'plug and play' style API, Radiate allows you to quickly test a multitude of evolutionary strategies and configurations.

* Traditional genetic algorithm implementation.
* Single & Multi-objective optimization support.
* Neuroevolution (graph-based representation - [evolving neural networks](http://www.scholarpedia.org/article/Neuroevolution)) support. Simmilar to [NEAT](https://nn.cs.utexas.edu/downloads/papers/stanley.ec02.pdf).
* Genetic programming support ([tree-based representation](https://en.wikipedia.org/wiki/Gene_expression_programming#:~:text=In%20computer%20programming%2C%20gene%20expression,much%20like%20a%20living%20organism.)) 
* Built-in support for parallelism.
* Extensive selection, crossover, and mutation operators with the ability to create custom ones.