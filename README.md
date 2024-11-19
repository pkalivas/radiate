# Radiate
### A Rust Library for Genetic Algorithms and Artificial Evolution

#### Major braeking changes as of 11/16/24 - v1.6.0. Readme as well as docs are still being put together and will improve. Check examples for current usages.

Radiate is a powerful Rust library designed for implementing genetic algorithms and artificial evolution techniques. It provides a flexible framework for creating, evolving, and optimizing solutions to complex problems using principles inspired by natural selection and genetics. This library is suitable for researchers, developers, and enthusiasts interested in evolutionary computation and optimization.

Large insperation for this library coming from other genetic algorithm libraries:
[Jenetics](https://github.com/jenetics/jenetics): A Java implementatino of GAs.
[genevo](https://github.com/innoave/genevo): Popular rust GA.
[radiate_legacy](https://github.com/pkalivas/radiate.legacy): Previous implemenation of this library with direct encoding.

### Features
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
* **Parallel Processing**: Utilize multi-threading capabilities to speed up the evolution process. Simply define the number of desired threads to process the fitness function on.
* **Flexible Fitness Functions**: Easily define and integrate custom fitness functions to evaluate individuals.

The implemenation of the genetic engine results in an extremely extensible and dynamic architecture. Mix and match any of these features togher or add new features and algorithms with minimal effort. Check radiate-extensions for additions.

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