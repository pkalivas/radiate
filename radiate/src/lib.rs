//! Radiate is a powerful Rust library designed for implementing genetic algorithms and artificial evolution techniques. It
//! provides a fast and flexible framework for creating, evolving, and optimizing solutions to complex problems using principles
//! inspired by natural selection and genetics. With an intuitive, 'plug and play' style API, Radiate allows you to quickly test a multitude of evolutionary strategies and configurations.
//!
//! * Traditional genetic algorithm implementation.
//! * Single & Multi-objective optimization support.
//! * Neuroevolution (graph-based representation - [evolving neural networks](http://www.scholarpedia.org/article/Neuroevolution)) support. Simmilar to [NEAT](https://nn.cs.utexas.edu/downloads/papers/stanley.ec02.pdf).
//! * Genetic programming support ([tree-based representation](https://en.wikipedia.org/wiki/Gene_expression_programming#:~:text=In%20computer%20programming%2C%20gene%20expression,much%20like%20a%20living%20organism.))
//! * Built-in support for parallelism.
//! * Extensive selection, crossover, and mutation operators with the ability to create custom ones.
//! * Support for custom encodings and decodings.
//! * Support for custom fitness functions.
//! * Support for custom termination conditions.
//!
//! See the git repo's [examples](https://github.com/pkalivas/radiate/tree/master/examples) directory for more examples.
//!
pub mod engines;

pub use engines::*;
