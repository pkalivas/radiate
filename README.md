# Radiate
### A Rust Library for Genetic Algorithms and Artificial Evolution

Radiate is a powerful Rust library designed for implementing genetic algorithms and artificial evolution techniques. It provides a flexible framework for creating, evolving, and optimizing solutions to complex problems using principles inspired by natural selection and genetics. This library is suitable for researchers, developers, and enthusiasts interested in evolutionary computation and optimization.

### Features
* **Genetic Algorithms**: Implement standard genetic algorithm operations such as selection, crossover, and mutation.
* **Customizable Codexes**: Define how individuals are represented and manipulated within the population.
* **Parallel Processing**: Utilize multi-threading capabilities to speed up the evolution process.
* **Flexible Fitness Functions**: Easily define and integrate custom fitness functions to evaluate individuals.
* **Extensible Architecture**: Add new features and algorithms with minimal effort.

### Basic Usages 
```rust
use radiate::engines::codexes::int_codex::IntCodex;
use radiate::engines::genetic_engine::GeneticEngine;
use radiate::engines::score::Score;
use radiate::{Elite, Tournament};

fn main() {
    let codex = IntCodex::new(1, 10, 0, 100).with_bounds(0, 100);

    let engine = GeneticEngine::from_codex(&codex)
        .population_size(150)
        .minimizing()
        .offspring_selector(Elite::new())
        .survivor_selector(Tournament::new(4))
        .fitness_fn(|genotype: Vec<Vec<i32>>| {
            Score::from_int(genotype.iter().fold(0, |acc, chromosome| {
                acc + chromosome.iter().fold(0, |acc, gene| acc + gene)
            }))
        })
        .build();

    let result = engine.run(|output| {
        println!("[ {:?} ]: {:?}", output.index, output.best.first().unwrap());
        output.score().as_int() == 0
    });

    println!("{:?}", result);
}
```
### Examples
The radiate-examples directory contains several examples demonstrating the capabilities of the library, including:
* **Min-Sum Problem**: An example of minimizing a sum of integers.
* **N-Queens Problem**: A classic problem in which the goal is to place N queens on a chessboard such that no two queens threaten each other.
* **Regression Graph**: An example of using genetic algorithms for regression analysis.