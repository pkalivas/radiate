
Radiate is inspired from a multitude of other genetic algorithm libraries, all of which have their own unique features and capabilities. Some of the most notable inspirations include:

* [carrot](https://github.com/liquidcarrot/carrot): An architecture-free neural network library built around neuroevolution built in javascript
* [Genevo](https://github.com/innoave/genevo): A Rust library which provides building blocks to run simulations of optimization and search problems using genetic algorithms (GA).
* [Sharpneat](https://github.com/colgreen/sharpneat): A C# library for evolutionary computation, primarily focused on neuroevolution and artificial neural networks
* [Jenetics](https://jenetics.io): A Genetic Algorithm, Evolutionary Algorithm, Grammatical Evolution, Genetic Programming, and Multi-objective Optimization library, written in modern day Java

## Configuration
```toml
[dependencies]
radiate = "1.2.5"
```

## Core Implementations

Genetic Engine

* The GeneticEngine is the central component orchestrating the genetic algorithm. 
    It manages the population, evaluates fitness, and handles selection, crossover, and mutation processes.
    It is designed to be flexible and extensible, allowing customization to fit specific problem requirements.

Codex

  * The Codex is responsible for encoding and decoding genetic information. It acts as a bridge between the problem space and the solution space, allowing the genetic algorithm to operate on abstract genetic representations while solving real-world problems.

Selectors

  * Selectors are used to choose individuals for reproduction and survival. They play a crucial role in determining the evolutionary pressure applied to the population. Different selection strategies, such as tournament and roulette selection, are implemented to cater to various optimization needs.

Alterers

  * Alterers include crossover and mutation operators that introduce genetic diversity and enable exploration of the solution space. 
  The library provides a variety of built-in alterers, and users can define custom ones to suit specific problem domains.

Fitness Function

  * The fitness function evaluates how well an individual solves the problem at hand. It is a critical component that guides the evolutionary process by assigning scores to individuals based on their performance.

## Extensions

For genetic programming Radiate offers a separate crate called [radiate-extenions](https://crates.io/crates/radiate-extensions) which extend the core library with additional features. Mainly it provides a [genetic programming](https://en.wikipedia.org/wiki/Genetic_programming#:~:text=In%20artificial%20intelligence%2C%20genetic%20programming,to%20the%20population%20of%20programs.) - probems that are represented as Trees (Expression Trees) or Graphs (NeuroEvolution). These offer powerful ways to solve complex problems.

```toml
[dependencies]
radiate-extensions = "0.1.2"
```

!!! note

    I'm currently working on the docs for these. If you are interested in using it, please refer to the git repo's [exampless](https://github.com/pkalivas/radiate/tree/master/radiate-examples) which include examples of both Tree and Graph based genetic programming.


## Example

!!! example "Hello, Radiate!"

    ```rust
    use radiate::*;

    fn main() {
        let target = "Hello, Radiate!";
        let codex = CharCodex::new(1, target.len());

        let engine = GeneticEngine::from_codex(&codex)
            .offspring_selector(BoltzmannSelector::new(4_f32))
            .survivor_selector(TournamentSelector::new(3))
            .alterer(alters![
                UniformMutator::new(0.01),
                UniformCrossover::new(0.5)
            ])
            .fitness_fn(|genotype: Vec<Vec<char>>| {
                Score::from_usize(genotype.into_iter().flatten().zip(target.chars()).fold(
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
            let best_as_string = output.best[0].iter().collect::<String>();
            println!("[ {:?} ]: {:?}", output.index, best_as_string);

            output.score().as_usize() == target.len()
        });

        println!("{:?}", result); // Should print "Hello, Radiate!"
    }
    ```