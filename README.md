<h1 align="center">Radiate</h1>
<p align="center">
  <img src="/docs/assets/radiate.png" height="100">
</p>

<div align="center">
  <img src="https://img.shields.io/github/check-runs/pkalivas/radiate/master" alt="master branch checks" />
  <img src="https://img.shields.io/crates/v/radiate" alt="Crates.io" />
  <img src="https://img.shields.io/pypi/v/radiate?color=blue" alt="pypi.org" />
  <img src="https://img.shields.io/crates/l/radiate" alt="Crates.io License" />
  <img src="https://img.shields.io/badge/evolution-genetics-default" alt="Static badge" />
</div>

___

For more details check radiate's [user guide](https://pkalivas.github.io/radiate/) or cargo [docs](https://docs.rs/radiate/latest/radiate/).


Radiate is a powerful library for implementing genetic algorithms and artificial evolution techniques. It provides a fast and flexible framework for creating, evolving, and optimizing solutions to complex problems using principles
inspired by natural selection and genetics. The core is written in Rust and is available for Python.
 
* Traditional genetic algorithm implementation.
* Single & Multi-objective optimization support.
* Neuroevolution (graph-based representation - [evolving neural networks](http://www.scholarpedia.org/article/Neuroevolution)) support. Simmilar to [NEAT](https://nn.cs.utexas.edu/downloads/papers/stanley.ec02.pdf).
* Genetic programming support ([tree-based representation](https://en.wikipedia.org/wiki/Gene_expression_programming#:~:text=In%20computer%20programming%2C%20gene%20expression,much%20like%20a%20living%20organism.)) 
* Built-in support for parallelism.
* Extensive selection, crossover, and mutation operators.