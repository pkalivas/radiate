# Radiate 

<figure markdown="span">
  ![Radiate](assets/radiate.png){ width="100" }
</figure>

<center>
![master branch checks][master_branch_checks] ![Crates.io][crates_link] ![Crates.io License][license] ![Static badge][static_evolution_badge]
</center>

[crates_link]: https://img.shields.io/crates/v/radiate
[master_branch_checks]: https://img.shields.io/github/check-runs/pkalivas/radiate/master
[license]: https://img.shields.io/crates/l/radiate
[static_evolution_badge]: https://img.shields.io/badge/evolution-genetics-default

Radiate is a powerful Rust library designed for implementing genetic algorithms and artificial evolution techniques. It
provides a fast and flexible framework for creating, evolving, and optimizing solutions to complex problems using principles
inspired by natural selection and genetics. With an intuitive, 'plug and play' style API, Radiate allows you to quickly test a multitude of evolutionary strategies and configurations, making it an ideal tool for both beginners and experienced practitioners in the field of evolutionary computation.

___
```toml
[dependencies]
radiate = "1.2.3"
```
___
* **Ease of Use**
  
    :    Intuitive API design allows users to easily configure and run genetic algorithms without needed to know the nuts and bolts of the complex operations underlying them.

* **Modular Design**
  
    :    The library architecture enables users to mix and match different components such as selection strategies, crossover methods, and mutation techniques to suit their specific needs.

* **Performance**

    :    Leveraging Rust's performance capabilities, Radiate ensures efficient execution of genetic operations, even for large populations and complex problem spaces. 

* **Flexibility**
  
    :    Out of the box support for a customizable genotypes and fitness functions, Radiate can be adapted to a wide range of problem domains, from optimization tasks to machine learning applications.


<!-- 
## Features

- **Genetic Algorithm Operations**: The `GeneticEngine` implements standard genetic algorithm operations such as selection, crossover, and mutation.
- **Custom Genome Representations**: Define how individuals are represented within your problem space using different codex implementations.
- **Modular Design**: Mix and match different components such as selection strategies, crossover methods, and mutation techniques to suit your specific needs.
- **Parallel Processing**: The `GeneticEngine` utilizes a thread pool for parallel evaluation of fitness functions and genetic operations. It can be configured to use multiple threads for faster execution.
- **Flexible Fitness Functions**: Easily define and integrate custom fitness functions to evaluate individuals. Each evaluation of the fitness function is performed in the thread pool.
- **Extensible Architecture**: The `GeneticEngine` is designed to be extensible, allowing users to add custom components like selection strategies, crossover methods, and mutation techniques with minimal effort.
- **Multi-Objective Optimization**: Supports multi-objective optimization for problems with N-objective functions.
- **Minimization and Maximization**: Supports both minimization and maximization problems by specifying the optimization goal in the fitness function.
- **Metrics**: Tracks and provides various metrics on the evolution process and internal operations.

## Parameters -->