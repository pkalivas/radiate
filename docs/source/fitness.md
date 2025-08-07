# Fitness Functions

___
Fitness functions are the core of any genetic algorithm - they define how well an individual performs and guide the evolution process. Radiate supports several different types of fitness functions, from your run of the mill functions to advanced techniques like novelty search and composite fitness functions.

The fitness function takes a decoded phenotype (the actual data structure) and returns a `Score` that represents how well that individual performs. The score can be a single value for single-objective optimization or multiple values for multi-objective problems. The result of your fitness function should reflect the quality of the individual in relation to the problem being solved. 

!!! note "**Fitness functions must return valid scores**"
    
    All fitness functions must return a `Score` object. Radiate automatically converts common types like `f32`, `i32`, `Vec<f32>`, etc. into `Score` objects. NaN values are not allowed and will cause a panic. The number of values returned by the fitness function must match your objectives. For example, if you have two objectives, your fitness function must return a `Score` with two values.

## Overview

| Fitness Function Type | Purpose | Use Case | Complexity |
|----------------------|---------|----------|------------|
| [Simple Functions](#simple-fitness-functions) | Basic optimization | Mathematical problems, benchmarks | Low |
| [Composite Functions](#composite-fitness-functions) | Multi-objective combination | Balancing multiple goals | Medium |
| [Novelty Search](#novelty-search) | Behavioral diversity | Exploration, avoiding local optima | High |

___

## Simple Fitness

Simple fitness functions are the most common type - they take a phenotype and return a single score value. These can be any function that evaluates how well an individual performs. Things like mathematical functions, benchmarks, or even custom evaluation logic can be used as simple fitness functions. Your run of the mill mathematical functions like Rastrigin, Sphere, or Ackley functions are great examples of simple fitness functions. They take a vector of floats and return a single float score.

!!! note ":fontawesome-brands-python: Python `numba`"

    For python, In come cases (primarily if you are decoding to a `np.array`) it is possible to compile your fitness function down to native C using [numba](https://numba.pydata.org). 
    In most cases with, this will result in your engine running as fast or almost as fast as rust. Check the examples page for an example using this method.

=== ":fontawesome-brands-python: Python"

    ```python
    import radiate as rd
    import math

    A = 10.0
    RANGE = 5.12
    N_GENES = 2

    def fitness_fn(x):
        value = A * N_GENES
        for i in range(N_GENES):
            value += x[i]**2 - A * math.cos((2.0 * 3.141592653589793 * x[i]))
        return value

    codec = rd.FloatCodec.vector(N_GENES, (-5.12, 5.12))
    engine = rd.GeneticEngine(codec, fitness_fn, objectives="min")
    ```

=== ":fontawesome-brands-rust: Rust"

    ```rust
    use radiate::*;

    fn rastrigin_function(genotype: Vec<f32>) -> f32 {
        let mut value = 10.0 * 2 as f32;
        for i in 0..N_GENES {
            value += genotype[i].powi(2) - 10.0 * (2.0 * std::f32::consts::PI * genotype[i]).cos();
        }

        value
    }

    let engine = GeneticEngine::builder()
        .codec(FloatCodec::vector(2, -5.12..5.12))
        .minimizing()
        .fitness_fn(rastrigin_function)
        .build();
    ```

---

## Composite Fitness

Composite fitness functions allow you to combine multiple objectives into a single weighted fitness score. This is useful when you have multiple goals that need to be balanced.

### Weighted

The `CompositeFitnessFn` combines multiple fitness functions with weights to create a single objective.

<!-- 
    ```python
    import radiate as rd

    def accuracy_objective(model: Model) -> float:
        return calculate_accuracy(model, test_data)

    def complexity_objective(model: Model) -> float:
        return model.complexity()  # Lower is better

    def efficiency_objective(model: Model) -> float:
        return model.inference_time()  # Lower is better

    # Create composite fitness function
    composite_fitness = rd.CompositeFitnessFn.new()
        .add_weighted_fn(accuracy_objective, 0.6)      # 60% weight on accuracy
        .add_weighted_fn(complexity_objective, 0.25)   # 25% weight on complexity
        .add_weighted_fn(efficiency_objective, 0.15)   # 15% weight on efficiency

    engine = rd.GeneticEngine(
        codec=rd.ModelCodec(),
        fitness_func=composite_fitness,
        objectives="max"  # We want to maximize the composite score
    )
    ```
-->

=== ":fontawesome-brands-python: Python"

    !!! warning ":construction: Under Construction :construction:"
        The composite fitness function is currently under construction and not yet available in the Python API.


=== ":fontawesome-brands-rust: Rust"

    ```rust
    use radiate::*;

    fn accuracy_objective(model: &MyModel) -> f32 {
        calculate_accuracy(model, &test_data)
    }

    fn complexity_objective(model: &MyModel) -> f32 {
        model.complexity()  // Lower is better
    }

    fn efficiency_objective(model: &MyModel) -> f32 {
        model.inference_time()  // Lower is better
    }

    // Create composite fitness function
    let composite_fitness = CompositeFitnessFn::new()
        .add_weighted_fn(accuracy_objective, 0.6)      // 60% weight on accuracy
        .add_weighted_fn(complexity_objective, 0.25)   // 25% weight on complexity
        .add_weighted_fn(efficiency_objective, 0.15);  // 15% weight on efficiency

    let engine = GeneticEngine::builder()
        .codec(model_codec)
        .maximizing()
        .fitness_fn(composite_fitness)
        .build();
    ```

**Key Features:**

- **Weighted combination**: Each objective has a weight that determines its importance
- **Normalized scoring**: Scores are weighted and averaged
- **Flexible objectives**: Can combine any number of fitness functions
- **Single objective**: Results in a single score for selection

### Equal Weight

You can also add fitness functions with equal weights using `add_fitness_fn()`.

<!--
```python
composite_fitness = rd.CompositeFitnessFn.new()
    .add_fitness_fn(accuracy_objective)    # Weight = 1.0
    .add_fitness_fn(complexity_objective)  # Weight = 1.0
    .add_fitness_fn(efficiency_objective)  # Weight = 1.0
```
-->

=== ":fontawesome-brands-python: Python"

    !!! warning ":construction: Under Construction :construction:"
        The composite fitness function is currently under construction and not yet available in the Python API.

=== ":fontawesome-brands-rust: Rust"

    ```rust
    let composite_fitness = CompositeFitnessFn::new()
        .add_fitness_fn(accuracy_objective)    // Weight = 1.0
        .add_fitness_fn(complexity_objective)  // Weight = 1.0
        .add_fitness_fn(efficiency_objective); // Weight = 1.0
    ```

---

## Novelty Search

Novelty search is an advanced technique that rewards individuals for being behaviorally different from previously seen solutions, rather than just being "better" in terms of fitness. This helps avoid local optima and promotes exploration of the solution space.

Novelty search works by:

1. **Behavioral Descriptors**: Each individual is described by a behavioral descriptor (e.g., output patterns, feature values)
2. **Archive**: Novel solutions are stored in an archive
3. **Distance Calculation**: Novelty is measured as the average distance to the k-nearest neighbors in the archive
4. **Threshold**: Solutions with novelty above a threshold are added to the archive

You can implement your own behavioral descriptors by implementing the `Novelty` trait. This allows you to define how individuals are described and how distances between behaviors are calculated. Or you can use the built-in descriptors below:

- **HammingDistance**: For binary or categorical data
- **EuclideanDistance**: For continuous data
- **CosineDistance**: For vector data
- **NeatDistance**: For NEAT-style networks (Graph only)

### Basic Novelty Search

=== ":fontawesome-brands-python: Python"

    ```python
    import radiate as rd

    class MyModelBehaviorDescriptor:
        def __init__(self, individual: List[float]):
            self.individual = individual

        def get_behavior_vector(self) -> List[float]:
            # some code that describes the behavior of a vector
            ... 

    # Define a behavioral descriptor
    def description(self, individual: List[float]) -> List[float]:
        # Return behavioral characteristics 
        descriptor = MyModelBehaviorDesciptor(individual)
        return descriptor.get_behvior_vecotr()
        
    # Create novelty search fitness function
    novelty_fitness = rd.NoveltySearch(
        behavior=descriptor,
        # can use any of the distance inputs. The engine will use this to 
        # determine how 'novel' an individual is compared to the other's in the 
        # archinve or population, ultimently resulting in the individuals fitness score.
        distance=rd.CosineDistance() 
        k=10,           # Number of nearest neighbors to consider
        threshold=0.1   # Novelty threshold for archive addition
        archive_size=1000 # defautls to 1000
    )

    engine = rd.GeneticEngine(
        codec=rd.ModelCodec(),
        fitness_func=novelty_fitness,
        # we always want to maximize novelty - however this is the default 
        # so its not necessary to define
        objective='max' 
    )
    ```

=== ":fontawesome-brands-rust: Rust"

    ```rust
    use radiate::*;

    // Define a behavioral descriptor
    struct BehaviorDescriptor;

    impl Novelty<Model> for BehaviorDescriptor {
        type Descriptor = Vec<f32>;

        fn description(&self, individual: &Model) -> Self::Descriptor {
            // Return behavioral characteristics (e.g., outputs on test cases)
            individual.get_behavior_vector()
        }

        fn distance(&self, a: &Self::Descriptor, b: &Self::Descriptor) -> f32 {
            // Calculate Euclidean distance between behaviors
            a.iter()
                .zip(b.iter())
                .map(|(x, y)| (x - y).powi(2))
                .sum::<f32>()
                .sqrt()
        }
    }

    // Create novelty search fitness function
    let novelty_fitness = NoveltySearch::new(
        BehaviorDescriptor,
        10,  // k: number of nearest neighbors
        0.1  // threshold: novelty threshold for archive addition
    )
    .with_archive_size(1000); // Optional: set archive size - default is 1000

    let engine = GeneticEngine::builder()
        .codec(model_codec)
        .maximizing()
        .fitness_fn(novelty_fitness)
        .build();
    ```

### Fitness-Based Novelty Search

You can use fitness scores directly as behavioral descriptors using `FitnessDescriptor`.

<!-- 
```python
def base_fitness(model: Model) -> float:
    return calculate_accuracy(model, test_data)

# Use fitness scores as behavioral descriptors
fitness_descriptor = rd.FitnessDescriptor(base_fitness)

novelty_fitness = rd.NoveltySearch(
    behavior=fitness_descriptor,
    k=10,
    threshold=0.05
)
```
-->

=== ":fontawesome-brands-python: Python"

    !!! warning ":construction: Under Construction :construction:"
        This function is currently under construction and not yet available in the Python API.

=== ":fontawesome-brands-rust: Rust"

    ```rust
    fn base_fitness(model: &Model) -> f32 {
        calculate_accuracy(model, &test_data)
    }

    // Use fitness scores as behavioral descriptors
    let fitness_descriptor = FitnessDescriptor::new(base_fitness);
    
    let novelty_fitness = NoveltySearch::new(
        fitness_descriptor,
        10,   // k
        0.05  // threshold
    );
    ```

### Combined Novelty and Fitness

You can combine novelty search with traditional fitness to get the benefits of both exploration and exploitation.

<!-- 
```python
def traditional_fitness(model: Model) -> float:
    return calculate_accuracy(model, test_data)

novelty_fitness = rd.NoveltySearch(
    behavior=rd.FitnessDescriptor(traditional_fitness),
    k=10,
    threshold=0.05
)

# Combine traditional fitness (70%) with novelty (30%)
combined_fitness = rd.CompositeFitnessFn.new()
    .add_weighted_fn(traditional_fitness, 0.7)
    .add_weighted_fn(novelty_fitness, 0.3)

engine = rd.GeneticEngine(
    codec=rd.ModelCodec(),
    fitness_func=combined_fitness,
    objectives="max"
)
```
-->

=== ":fontawesome-brands-python: Python"

    !!! warning ":construction: Under Construction :construction:"
        This function is currently under construction and not yet available in the Python API.

=== ":fontawesome-brands-rust: Rust"

    ```rust
    fn traditional_fitness(model: &Model) -> f32 {
        calculate_accuracy(model, &test_data)
    }

    let novelty_fitness = NoveltySearch::new(
        FitnessDescriptor::new(traditional_fitness),
        10,   // k
        0.05  // threshold
    );

    // Combine traditional fitness (70%) with novelty (30%)
    let combined_fitness = CompositeFitnessFn::new()
        .add_weighted_fn(traditional_fitness, 0.7)
        .add_weighted_fn(novelty_fitness, 0.3);

    let engine = GeneticEngine::builder()
        .codec(model_codec)
        .maximizing()
        .fitness_fn(combined_fitness)
        .build();
    ```

---

## Best Practices

### Choosing the Right Fitness Function Type

1. **Simple Functions**: Use for straightforward optimization problems
2. **Composite Functions**: Use when you have multiple objectives that can be weighted
3. **Novelty Search**: Use when you need to explore diverse solutions or avoid local optima and don't care much about the fitness score

### Performance Considerations

1. **Novelty Search**: More computationally expensive due to distance calculations and archive management
2. **Composite Functions**: Slight overhead from multiple function evaluations
3. **Archive Size**: Larger archives in novelty search provide better diversity but use more memory
4. **Distance Calculations**: Choose efficient distance metrics for novelty search

### Parameter Tuning

1. **Novelty Threshold**: Lower values add more solutions to archive, higher values are more selective
2. **K-Nearest Neighbors**: Higher k provides more stable novelty scores but is more expensive
3. **Weights in Composite Functions**: Balance objectives based on their relative importance
4. **Archive Size**: Balance memory usage with diversity preservation

### Common Patterns

1. **Fitness + Novelty**: Combine traditional fitness with novelty for balanced exploration/exploitation
2. **Multi-Objective Composite**: Use composite functions to handle multiple conflicting objectives
3. **Behavioral Descriptors**: Use domain-specific behavioral characteristics for novelty search
