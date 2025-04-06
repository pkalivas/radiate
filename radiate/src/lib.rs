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
pub mod alterers;
pub mod audit;
pub mod builder;
pub mod codexes;
pub mod context;
pub mod domain;
pub mod engine;
pub mod genome;
pub mod objectives;
pub mod params;
pub mod problem;
pub mod replace;
pub mod selectors;
pub mod stats;

pub use alterers::*;
pub use audit::*;
pub use builder::*;
pub use codexes::{
    BitCodex, CharCodex, Codex, FloatCodex, FnCodex, IntCodex, PermutationCodex, SubSetCodex,
};
pub use context::*;
pub use domain::*;
pub use engine::*;
pub use genome::*;
pub use objectives::{Front, Objective, Optimize, Score, pareto};
pub use params::*;
pub use problem::*;
pub use replace::*;
pub use selectors::*;
pub use stats::*;
