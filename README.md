# Radiate
### A Rust Library for Genetic Algorithms and Artificial Evolution

#### Major braeking changes as of 11/16/24 - v1.6.0. Readme is still being put together and will improve. Check examples of current usages.

Radiate is a powerful Rust library designed for implementing genetic algorithms and artificial evolution techniques. It provides a flexible framework for creating, evolving, and optimizing solutions to complex problems using principles inspired by natural selection and genetics. This library is suitable for researchers, developers, and enthusiasts interested in evolutionary computation and optimization.

### Features
* **Genetic Algorithms**: Implement standard genetic algorithm operations such as selection, crossover, and mutation.
* **Customizable Codexes**: Define how individuals are represented.
* **Parallel Processing**: Utilize multi-threading capabilities to speed up the evolution process.
* **Flexible Fitness Functions**: Easily define and integrate custom fitness functions to evaluate individuals.
* **Extensible Architecture**: Add new features and algorithms with minimal effort. Check radiate-extensions.

### Basic Usage
Evolve a string of characters to match the target (Chicago, IL)
```rust
use radiate::*;

fn main() {
    let target = "Chicago, IL";
    let codex = CharCodex::new(1, target.len());

    let engine = GeneticEngine::from_codex(&codex)
            .offspring_selector(Elite::new())
            .survivor_selector(Tournament::new(3))
            .alterer(vec![Alterer::Mutator(0.1), Alterer::UniformCrossover(0.5)])
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
### Examples
The radiate-examples directory contains several examples demonstrating the capabilities of the library, including:
* **Min-Sum Problem**: An example of minimizing a sum of integers.
* **N-Queens Problem**: A classic problem in which the goal is to place N queens on a chessboard such that no two queens threaten each other.
* **Regression Graph**: An example of using genetic algorithms for regression analysis.