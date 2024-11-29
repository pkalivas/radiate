
___
Configuring genetic algorithms can be a complex task, requiring careful consideration of various parameters and settings to achieve optimal performance. The `GeneticEngine` struct in Radiate provides a convenient way to configure all these parameters in a single place, making it easy to experiment with different settings and strategies.

The `GeneticEngine` will default as many parameters as it can, but it is recommended to set the parameters that are relevant to your problem space. To get the engine off the ground there are two required parameters:

* [`codex`](codex.md)
* `fitness_fn`

Without these there is no way to represent the problem space and no way to evaluate the fitness of the individuals in the population.

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
    ???+ warning "Required"

        | Default | Type |
        |---------|------|
        | `None` | `Box<dyn Codex<C: Chromosome, T>` |

* `fitness_fn`

    :   The fitness function used to evaluate the fitness of individuals in the population. The fitness function takes a `Genotype` as input and returns a `Score` representing the fitness value of the individual. The genetic algorithm uses the fitness function to evaluate the quality of individuals and guide the evolutionary process.
    ???+ warning "Required"

        | Default | Type |
        |---------|------|
        | `None` | `Box<dyn Fn(T) -> Score + Send + Sync>` |
