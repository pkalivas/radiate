
___
Configuring genetic algorithms can be a complex task, requiring careful consideration of various parameters and settings to achieve optimal performance. The `GeneticEngine` struct in Radiate provides a convenient way to configure all these parameters in a single place, making it easy to experiment with different settings and strategies.

The `GeneticEngine` will default as many parameters as it can, but it is recommended to set the parameters that are relevant to your problem space. To get the engine off the ground there are two required parameters:

* [`codec`](codec.md)
* `fitness_fn`

Without these there is no way to represent the problem space and no way to evaluate the fitness of the individuals in the population.

By default, the engine will use a parameter set defined as such:
```rust
GeneticEngineBuilder {
    population_size: 100,
    max_age: 20,
    offspring_fraction: 0.8,
    front_range: 800..900,
    thread_pool: ThreadPool::new(1),
    objective: Objective::Single(Optimize::Maximize),
    survivor_selector: Arc::new(TournamentSelector::new(3)),
    offspring_selector: Arc::new(RouletteSelector::new()),
    replacement_strategy: Arc::new(EncodeReplace),
    audits: vec![Arc::new(MetricAudit)],
    alterers: Vec::new(),
    codec: None,
    population: None,
    fitness_fn: None,
    problem: None,
    front: None,
}
```

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
        | 20 | usize | 1..=usize::MAX |

* `offspring_fraction`

    :   The fraction of the population that will be replaced by offspring in each generation. A value of 0.5 means that half of the population will
        be replaced by offspring in each generation. A higher value can lead to faster convergence but may reduce diversity in the population.
    ??? info "Optional"

        | Default | Type | Range |
        |---------|------|-------|
        | 0.8 | f32 | 0.0..=1.0 |


* `front_size`

    :   Only used in multi-objective optimization. The range of individuals to collect within a pareto front. The front will collect up to the end range and if the range is exceeded, the front will filter out individuals with the least dominating score until it settles within the provided range.
    
    ??? info "Optional"

        | Default | Type | Range |
        |---------|------|-------|
        | 800..900 | Range<usize> | 1..=`usize::MAX`|

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

* `mutator` 

    :   The genetic operator used to mutate individuals in the population. This allows for the input of a single specific mutator to the engine. Unlike the `alters` parameter above, which allows for mulitple operators of either mutation or crossover, this parameter allwos just a single mutator.

    ??? info "Optional"

        | Default | Type |
        |---------|------|
        | `UniformMutator::new(0.01)` | `Option<Box<dyn Alter<C: Chromosome>>` |

* `mutators`

    :   The genetic operators used to mutate individuals in the population. This allows for the input of multiple specific mutators to the engine. Unlike the `alters` parameter above, which allows for both mutation and crossover operators, this parameter allwos just mutators.

    ??? info "Optional"

        | Default | Type |
        |---------|------|
        | `UniformMutator::new(0.01)` | `Vec<Box<dyn Alter<C: Chromosome>>` |

* `crossover` 

    :   The genetic operator used to crossover individuals in the population. This allows for the input of a single specific crossover operator to the engine. Unlike the `alters` parameter above, which allows for multiple operators of either mutation or crossover, this parameter allwos just a single crossover.

    ??? info "Optional"

        | Default | Type |
        |---------|------|
        | `UniformCrossover::new(0.5)` | `Option<Box<dyn Alter<C: Chromosome>>` |

* `crossovers`
  
    :   The genetic operators used to crossover individuals in the population. This allows for the input of multiple specific crossover operators to the engine. Unlike the `alters` parameter above, which allows for both mutation and crossover operators, this parameter allwos just crossovers.

    ??? info "Optional"

        | Default | Type |
        |---------|------|
        | `UniformCrossover::new(0.5)` | `Vec<Box<dyn Alter<C: Chromosome>>` |

* `audits`

    :   An audit function which will be used to inject custom metrics into the engine's context during evolution. This has proven useful when tracking progress of the engine or attempting to understand the behavior of the genetic algorithm.

    ??? info "Optional"

        | Default | Type |
        |---------|------|
        | `None` | `Vec<Arc<dyn Audit<C>>>` |

* `replacement_strategy`

    :   During the evolution process certain individuals can reach the max age parameter or be deemed invalid by the chromosome or genes. Because of this, those individuals need to be replaced within the populationl. This parameter handles that. By default, the engine will use the provided `Codec` to simply encode a new individual, however custom replacements can be created and used. This can include strategies like `EncodeReplace`, `PopulationSampleReplace`, or other custom strategies.
    ??? info "Optional"

        | Default | Type |
        |---------|------|
        | `Elitism::new(1)` | `Arc<dyn ReplacementStrategy<C>>` |

* `population`

    :   The initial population of individuals to start the genetic algorithm. If no initial population is provided, the genetic algorithm will generate a random initial population based on the codec.
    ??? info "Optional"

        | Default | Type |
        |---------|------|
        | `None` | `Option<Vec<Phenotype<C: Chromosome>>` |

* `codec`

    :   The codec that defines how individuals are represented in the genetic algorithm. The codec is responsible for encoding and decoding the genetic information of individuals, allowing the genetic algorithm to operate on the genetic information in a meaningful way.
    ???+ warning "Required"

        | Default | Type |
        |---------|------|
        | `None` | `Box<dyn Codec<C: Chromosome, T>` |

* `fitness_fn`

    :   The fitness function used to evaluate the fitness of individuals in the population. The fitness function takes a `Genotype` as input and returns a `Score` representing the fitness value of the individual. The genetic algorithm uses the fitness function to evaluate the quality of individuals and guide the evolutionary process.
    ???+ warning "Required"

        | Default | Type |
        |---------|------|
        | `None` | `Box<dyn Fn(T) -> Score + Send + Sync>` |
