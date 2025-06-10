#  

<center>
    <h1>Radiate</h1>
</center>
<figure markdown="span">
  ![Radiate](assets/radiate.png){ width="100" }
</figure>

<div align="center">
    <a href="https://github.com/pkalivas/radiate/actions/workflows/unit-tests.yml">
        <img src="https://img.shields.io/github/check-runs/pkalivas/radiate/master" alt="master branch checks" />
    </a>
    <a href="https://crates.io/crates/radiate">
        <img src="https://img.shields.io/crates/v/radiate" alt="Crates.io" />
    </a>
    <a href="https://pypi.org/project/radiate/">
        <img src="https://img.shields.io/pypi/v/radiate?color=blue" alt="pypi.org" />
    </a>
    <a href="https://github.com/pkalivas/radiate?tab=MIT-1-ov-file">
        <img src="https://img.shields.io/crates/l/radiate" alt="Crates.io License" />
    </a>
    <a href="">
        <img src="https://img.shields.io/badge/evolution-genetics-default" alt="Static badge" />
    </a>
</div>

___

Radiate is a powerful library for implementing genetic algorithms and artificial evolution techniques. It provides a fast and flexible framework for creating, evolving, and optimizing solutions to complex problems using principles
inspired by natural selection and genetics. The core is written in Rust and is available for Python.

<div class="grid cards" markdown>
    
-   **Ease of use** :material-thumb-up:{ .right }

    ---

    Intuitive API design allows users to easily configure and run genetic algorithms without needed to know the nuts and bolts of the complex operations underlying them.

-   **Modular Design** :material-draw:{ .right }

    ---

    The library architecture enables users to mix and match different components such as selection strategies, crossover methods, and mutation techniques to suit their specific needs.

-   **Performance** :material-rocket-launch:{ .right }

    ---

    Between Rust's performance capabilities, a multi-threaded architecture, and hours spent optimizing code, Radiate ensures efficient execution of genetic operations, even for large populations and complex problem spaces. 

-   **Flexibility** :material-domain:{ .right }

    ---

    Out of the box support for a customizable genotypes and fitness functions, Radiate can be adapted to a wide range of problem domains, from single and multi optimization tasks to neuroevolution and machine learning applications.

</div>

## Key Features

- **Genetic Engine**: The central component orchestrating the genetic algorithm. It manages the ecosystem, evaluates fitness, and handles selection, crossover, and mutation processes. It is designed to be flexible and extensible, allowing customization to fit specific optimization requirements.

- **Codec**: Responsible for encoding and decoding genetic information. It acts as a bridge between the problem space and the solution space, allowing the genetic algorithm to operate on abstract genetic representations while solving real-world problems.

- **Selectors**: Used to choose individuals for reproduction and survival. They play a crucial role in determining the evolutionary pressure applied to the population.

- **Alterers**: Crossover and mutation operators that introduce genetic diversity and enable exploration of the solution space. The library provides a variety of built-in alterers.

- **Fitness Function**: Evaluates how well an individual solves the problem at hand. It is a critical component that guides the evolutionary process by assigning scores to individuals based on their performance.

## Example

This simple maximizing problem demonstrates how to use Radiate to solve a string matching problem, where the goal is to evolve a genotype of chars to match a target string.

!!! example "Hello, Radiate!"

    === ":fontawesome-brands-python: Python"

        ```python
        from typing import List
        import radiate as rd 
        
        target = "Hello, Radiate!"

        def fitness_func(x: List[str]) -> int:
            return sum(1 for i in range(len(target)) if x[i] == target[i])

        engine = rd.GeneticEngine(
            codec=rd.CharCodec.vector(len(target)),
            fitness_func=fitness_func,
            objectives='max',
            offspring_selector=rd.BoltzmannSelector(4),
        )

        result = engine.run(rd.ScoreLimit(len(target)))

        print(result)
        ```

    === ":fontawesome-brands-rust: Rust"
    
        ```rust
        use radiate::*;

        fn main() {
            let target = "Hello, Radiate!";
            let codec = CharCodec::vector(target.len());

            let mut engine = GeneticEngine::builder()
                .codec(codec)
                .offspring_selector(BoltzmannSelector::new(4_f32))
                .fitness_fn(|geno: Vec<char>| {
                    geno.into_iter().zip(target.chars()).fold(
                        0,
                        |acc, (allele, targ)| {
                            if allele == targ { acc + 1 } else { acc }
                        },
                    )
                })
                .build();

            let result = engine.run(|ctx| {
                let best_as_string = ctx.best.iter().flatten().collect::<String>();
                println!("[ {:?} ]: {:?}", ctx.index, best_as_string);

                ctx.score().as_usize() == target.len()
            });

            println!("{:?}", result); 
        }
        ```

## Outside Inspirations

Radiate is inspired from a multitude of other genetic algorithm libraries, all of which have their own unique features and capabilities. Some of the most notable inspirations include:

* [carrot](https://github.com/liquidcarrot/carrot): An architecture-free neural network library built around neuroevolution built in javascript
* [Genevo](https://github.com/innoave/genevo): A Rust library which provides building blocks to run simulations of optimization and search problems using genetic algorithms (GA).
* [Sharpneat](https://github.com/colgreen/sharpneat): A C# library for evolutionary computation, primarily focused on neuroevolution and artificial neural networks
* [Jenetics](https://jenetics.io): A Genetic Algorithm, Evolutionary Algorithm, Grammatical Evolution, Genetic Programming, and Multi-objective Optimization library, written in modern day Java