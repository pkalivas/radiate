
___
The `GeneticEngine` is the core component of the Radiate library's genetic algorithm implementation. It orchestrates the entire evolutionary process, managing the population of individuals, evaluating their fitness, and applying genetic operations such as selection, crossover, and mutation to evolve solutions over generations.
Its designed to be fast, flexible, and extensible, allowing users to customize various aspects of the genetic algorithm to suit their specific needs. It provides a high-level abstraction that simplifies the setup and execution of genetic algorithms.

## Parameters
___
* `population_size`

    :   The number of individuals in the population. A larger population size can help the genetic algorithm explore a larger solution space but may require more computational resources.

    ??? info "Optional"

        | Default | Type | Range |
        |---------|------|-------|
        | 100 | usize | 1..=usize::MAX |

* `max_age`

    :   The maximum number of generations an individual can live before being replaced by a new offspring. Sometimes a `Phenotype` can survive for many generations without being replaced by a new offspring. This can lead to stagnation in the population and prevent the genetic algorithm from exploring new solutions. The `Max Age` parameter helps to prevent this by limiting the lifespan of individuals in the population. If an individual reaches the maximum age, it is replaced by a newly encoded individual.

    ??? info "Optional"

        | Default | Type | Range |
        |---------|------|-------|
        | 25 | usize | 1..=usize::MAX |

* `offspring_fraction`

    :   The fraction of the population that will be replaced by offspring in each generation. A value of 0.5 means that half of the population will
        be replaced by offspring in each generation. A higher value can lead to faster convergence but may reduce diversity in the population.
    ??? info "Optional"

        | Default | Type | Range |
        |---------|------|-------|
        | 0.8 | f32 | 0.0..=1.0 |


* `min_front_size`

    :   Only used in multi-objective optimization. The minimum number of individuals that will be kept in a pareto front.
    ??? info "Optional"

        | Default | Type | Range |
        |---------|------|-------|
        | 1000 | usize | 1..=`max_front_size`|

* `max_front_size`

    :   Only used in multi-objective optimization. The maximum number of individuals that will be kept in a pareto front.
    ??? info "Optional"

        | Default | Type | Range |
        |---------|------|-------|
        | 1500 | usize | `min_front_size`..=usize::MAX |

* `num_threads`

    :   The number of threads used by the genetic algorithm to evaluate fitness functions and perform genetic operations in parallel. A higher number of threads can speed up the evolutionary process by allowing multiple individuals to be evaluated simultaneously. However, using too many threads can lead to resource contention and reduce performance.
    ??? info "Optional"

        | Default | Type | Range |
        |---------|------|-------|
        | 1 | usize | 1..=usize::MAX |

* `objective`

    :   The optimization goal of the genetic algorithm. It can be set to either `Maximize` or `Minimize` depending on the problem being solved. For multi-objective optimization, the objective can be set to `Objective::Multi(Vec<Optimize>)`.
    ??? info "Optional"

        | Default | Type |
        |---------|------|
        | `Objective::Single(Optimize::Maximize)` | Objective |

* `survivor_selector`

    :   The selection strategy used to choose individuals from the population to survive to the next generation. The survivor selector is responsible for selecting individuals based on their fitness values and other criteria.
    ??? info "Optional"

        | Default | Type |
        |---------|------|
        | `TournamentSelector::new(3)` | `Box<dyn Selector<C: Chromosome>` |

* `offspring_selector`

    :   The selection strategy used to choose individuals from the population to produce offspring in each generation. The offspring selector is responsible for selecting individuals based on their fitness values and other criteria.
    ??? info "Optional"

        | Default | Type |
        |---------|------|
        | `RouletteSelector::new()` | `Box<dyn Selector<C: Chromosome>` |

* `alters`

    :   The genetic operators used to alter the genetic information of individuals chosen from the `offspring_selector`. Genetic operators include mutation, crossover, and other operations that modify the genetic information of individuals to create new offspring. Common genetic operators include `UniformMutator`, `UniformCrossover`, and `SwapMutator`.
    ??? info "Optional"

        | Default | Type |
        |---------|------|
        | `UniformMutator::new(0.01)`, `UniformCrossover::new(0.5)` | `Vec<Box<dyn Alter<C: Chromosome>>` |

* `population`

    :   The initial population of individuals to start the genetic algorithm. If no initial population is provided, the genetic algorithm will generate a random initial population based on the codex.
    ??? info "Optional"

        | Default | Type |
        |---------|------|
        | `None` | `Option<Vec<Phenotype<C: Chromosome>>` |

* `codex`

    :   The codex that defines how individuals are represented in the genetic algorithm. The codex is responsible for encoding and decoding the genetic information of individuals, allowing the genetic algorithm to operate on the genetic information in a meaningful way.
    ??? info "Required"

        | Default | Type |
        |---------|------|
        | `None` | `Box<dyn Codex<C: Chromosome, T>` |

* `fitness_fn`

    :   The fitness function used to evaluate the fitness of individuals in the population. The fitness function takes a `Genotype` as input and returns a `Score` representing the fitness value of the individual. The genetic algorithm uses the fitness function to evaluate the quality of individuals and guide the evolutionary process.
    ??? info "Required"

        | Default | Type |
        |---------|------|
        | `None` | `Box<dyn Fn(T) -> Score + Send + Sync>` |

## Setup

The `GeneticEngine` allows you to build a parameter set it then uses during execution. The `GeneticEngineParams` struct is used to configure the genetic algorithm's behavior and parameters. You can customize various aspects of the genetic algorithm by setting the appropriate fields in the `GeneticEngineParams` struct.

Here's an example of setting up a `GeneticEngine` with custom parameters:

```rust
use radiate::*;

// Define the codex for the problem space - e.g., BitCodex for a vec of bits. This is 
// always required.
let codex = BitCodex::new(1, 8); // 1 chromosome with 8 genes

let engine = GeneticEngine::from_codex(&codex)
    .population_size(100)
    .minimizing()
    .max_age(50)
    .offspring_fraction(0.5)
    .num_threads(4)
    .offspring_selector(RouletteSelector::new())
    .survivor_selector(TournamentSelector::new(3))
    .alterer(alters![
        UniformMutator::new(0.01),
        UniformCrossover::new(0.5)
    ])
    .fitness_fn(|genotype: Vec<Vec<bool>>| {
        let sum: usize = genotype.iter().map(|chromosome| {
            chromosome.iter().map(|gene| if *gene { 1 } else { 0 }).sum::<usize>()
        }).sum();

        Score::from_usize(sum)
    })
    .build();
```