<h1 align="center">Radiate</h1>
<p align="center">
  <img src="/docs/assets/radiate.png" height="100" width="60" alt="Radiate Logo" />
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
* Opt-in speciation for maintaining diversity.
* Novelty search support.
* First-class metric tracking.

--- 
## Installation
### Rust
Add this to your `Cargo.toml`:
```toml
[dependencies]
radiate = { version = "1.2.19", features = ["x"] }
``` 
### Python
```bash
pip install radiate # --or-- uv add radiate
```

---
## Building from source
```bash
git clone https://github.com/pkalivas/radiate.git
cd radiate
```
The core build options are below, there are a few others that can be found through the `make help` command.

* `make build` to build both Rust and Python packages in develop mode
  * add `ARGS="--release"` to build both packages in release mode
  * add `PY=3.x` to build python package for specific python version (e.g. `PY=3.12`, `PY=3.13t` for free-threading interpreter)
  
* `make test-rs` to run tests for rust
* `make test-py` to run tests for python package

