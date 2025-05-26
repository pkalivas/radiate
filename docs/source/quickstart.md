
## Installing Radiate

=== ":fontawesome-brands-python: Python"

    ```bash
    pip install radiate
    ```

=== ":fontawesome-brands-rust: Rust"
    ```shell
    cargo add radiate -F gp

    # Or Cargo.toml
    [dependencies]
    radiate = { version = "x", features = ["gp"] }
    ```

## Core Implementations

Genetic Engine

* The `GeneticEngine` is the central component orchestrating the genetic algorithm. 
    It manages the population, evaluates fitness, and handles selection, crossover, and mutation processes.
    It is designed to be flexible and extensible, allowing customization to fit specific optimization requirements.

Codex

  * The Codex is responsible for encoding and decoding genetic information. It acts as a bridge between the problem space and the solution space, allowing the genetic algorithm to operate on abstract genetic representations while solving real-world problems.

Selectors

  * Selectors are used to choose individuals for reproduction and survival. They play a crucial role in determining the evolutionary pressure applied to the population. Different selection strategies, such as tournament and roulette selection, are implemented to cater to various optimization needs.

Alterers

  * Alterers include crossover and mutation operators that introduce genetic diversity and enable exploration of the solution space. 
  The library provides a variety of built-in alterers, and users can define custom ones to suit specific problem domains.

Fitness Function

  * The fitness function evaluates how well an individual solves the problem at hand. It is a critical component that guides the evolutionary process by assigning scores to individuals based on their performance.

## Features

Radiate offers a feature called `gp` which extend the core library with additional features. Mainly it provides data structures and algorithms which facilitate [Genetic Programming](https://en.wikipedia.org/wiki/Genetic_programming#:~:text=In%20artificial%20intelligence%2C%20genetic%20programming,to%20the%20population%20of%20programs.) (GP) - probems that are represented as Trees or Graphs. This is a powerful extension to the core library, allowing users to tackle even more complex or unique problems. 

To use it, add the following to your `Cargo.toml`:

```toml
[dependencies]
radiate = { version = "1.2.12", features = ["gp"] }
```

!!! note

    As of 3/5/2025

    I'm currently working on the docs for these. If you are interested in using it, please refer to the git repo's [examples](https://github.com/pkalivas/radiate/tree/master/examples) which include examples of both Tree and Graph based genetic programming.


## Example

!!! example "Hello, Radiate!"

    ```rust
    use radiate::*;

    fn main() {
        let target = "Hello, Radiate!";
        let codex = CharCodex::new(1, target.len());

        let engine = GeneticEngine::from_codex(codex)
            .offspring_selector(BoltzmannSelector::new(4_f32)) // optional
            .fitness_fn(|geno: Vec<Vec<char>>| {
                geno.into_iter()
                    .flatten()
                    .zip(target.chars())
                    .fold(
                        0,
                        |acc, (geno, targ)| {
                            if geno == targ {
                                acc + 1
                            } else {
                                acc
                            }
                        },
                    )
            })
            .build();

        let result = engine.run(|ctx| {
            let best_as_string = ctx.best.iter().flatten().collect::<String>();
            println!("[ {:?} ]: {:?}", ctx.index, best_as_string);

            ctx.score().as_usize() == target.len()
        });

        // prints the final `EvolutionContext` which contains the final population, best individual,
        // the number of generations (index), best score, and the `MetricSet` (a collection of 
        // evolution metrics the engine maintains throughout the run)
        println!("{:?}", result); 
    }
    ```