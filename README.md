# <p align="center">Radiate</p>

![master branch checks][br_ck] ![Crates.io][cl] ![Crates.io License][li]


[cl]: https://img.shields.io/crates/v/radiate
[br_ck]: https://img.shields.io/github/check-runs/pkalivas/radiate/master
[li]: https://img.shields.io/crates/l/radiate



#### Major braeking changes as of 11/16/24 - v1.6.0. Readme as well as docs are still being put together and will improve. Check examples for current usages.

Radiate is a powerful Rust library designed for implementing genetic algorithms and artificial evolution techniques. It provides a flexible framework for creating, evolving, and optimizing solutions to complex problems using principles inspired by natural selection and genetics. This library is suitable for researchers, developers, and enthusiasts interested in evolutionary computation and optimization.

---

Large insperation for this library coming from other genetic algorithm libraries:
[Jenetics](https://github.com/jenetics/jenetics): A Java implementatino of GAs.
[genevo](https://github.com/innoave/genevo): Popular rust GA.
[radiate_legacy](https://github.com/pkalivas/radiate.legacy): Previous implemenation of this library with direct encoding.

## Usage
Add to cargo.toml
```toml
[dependencies]
radiate = "1.2.2"
```

## Features
* **Genetic Algorithms**: Implement standard genetic algorithm operations such as selection, crossover, and mutation.
  * [Selectors](https://en.wikipedia.org/wiki/Selection_(genetic_algorithm)#:~:text=Boltzmann%20selection,-In%20Boltzmann%20selection&text=The%20temperature%20is%20gradually%20lowered,the%20appropriate%20degree%20of%20diversity.):
      1. Boltzmann
      2. Elitism 
      3. Rank
      4. Roulette
      5. Tournament
   * [Crossovers](https://en.wikipedia.org/wiki/Crossover_(genetic_algorithm)):
      1. Singlepoint
      2. Multipoint
      3. Uniform
      4. Mean (average between two numerical genes)
    * [Mutations](https://en.wikipedia.org/wiki/Mutation_(genetic_algorithm)):
      1. Mutator (random gene replacement)
      2. Swap 
      3. Numeric
* **Customizable Codexes**: Define how individuals are represented.
  * Each ```Genotype``` can be thought of as a matrix of ```Genes```. Each row being a ```Chromosoome```. This means the decoding of a ```Genotype``` reults in a ```Vec<Vec<T>>```. For example, a ```Genotype``` of ```FloatGene``` decodes to ```Vec<Vec<f32>>```
* **Parallel Processing**: The ```GeneticEngine``` has a thread pool it pushes work to when applicable. Simply define the number of desired threads.
* **Flexible Fitness Functions**: Easily define and integrate custom fitness functions to evaluate individuals. Each evaluation of the fitness function if evalued in a thread pool.

The implemenation of the ```GeneticEngine``` results in an extremely extensible and dynamic architecture. Mix and match any of these features together or add new features and algorithms with minimal effort. Check [radiate-extensions](https://github.com/pkalivas/radiate/tree/master/radiate-extensions) for additions.

## Basic Usage
Evolve a string of characters to match the target (Chicago, IL)
```rust
use radiate::*;

fn main() {
    let target = "Chicago, IL";
    let codex = CharCodex::new(1, target.len());

    let engine = GeneticEngine::from_codex(&codex)
            .offspring_selector(RankSelector::new())
            .survivor_selector(TournamentSelector::new(3))
            .alterer(vec![
                Alterer::Mutator(0.01),
                Alterer::UniformCrossover(0.5)
            ])
            .fitness_fn(|genotype: String| {
                Score::from_usize(genotype.chars().zip(target.chars()).fold(
                    0,
                    |acc, (geno, targ)| {
                        if geno == targ {
                            acc + 1
                        } else {
                            acc
                        }
                    },
                ))
            })
            .build();

    let result = engine.run(|output| {
        println!("[ {:?} ]: {:?}", output.index, output.best);

        output.score().as_usize() == target.len()
    });

    println!("{:?}", result);
}
```
## Workflow
TODO - write out the general GA workflow provided by the library.

## Examples
The radiate-examples directory contains several examples demonstrating the capabilities of the library, including:
* **[Min-Sum](https://github.com/pkalivas/radiate/blob/master/radiate-examples/min-sum/src/main.rs)**: An example of minimizing a sum of integers.
* **[N-Queens](https://github.com/pkalivas/radiate/blob/master/radiate-examples/nqueens/src/main.rs)**: A classic problem in which the goal is to place N queens on a chessboard such that no two queens threaten each other.
* **[Knapsack](https://github.com/pkalivas/radiate/blob/master/radiate-examples/knapsack/src/main.rs)**: Another classic problem for evolutionary algorithms.
* **[Regression Graph](https://github.com/pkalivas/radiate/blob/master/radiate-examples/regression-graph/src/main.rs)**: Evolve a ```Graph<f32>``` (essentially a graph based neural network) for regression analysis.
* **[Simple Memory Graph](https://github.com/pkalivas/radiate/blob/master/radiate-examples/simple-memory-graph/src/main.rs)**: Evolve a ```Graph<f32>``` (Neural Network) using recurrent connections for Neural Network based memory.
* **[XOR Graph](https://github.com/pkalivas/radiate/blob/master/radiate-examples/xor-graph/src/main.rs)**: Evolve a ```Graph<f32>``` to solve the classic [XOR problem](https://dev.to/jbahire/demystifying-the-xor-problem-1blk).