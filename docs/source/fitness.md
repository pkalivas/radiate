# Fitness Functions

___
Fitness functions are the core of any genetic algorithm - they define how well an individual performs and guide the evolution process. Radiate supports several different types of fitness functions, from simple mathematical functions to advanced techniques like novelty search and composite fitness functions.

The fitness function takes a decoded phenotype (the actual data structure) and returns a `Score` that represents how well that individual performs. The score can be a single value for single-objective optimization or multiple values for multi-objective problems.

!!! note "**Fitness functions must return valid scores**"
    
    All fitness functions must return a `Score` object. Radiate automatically converts common types like `f32`, `i32`, `Vec<f32>`, etc. into `Score` objects. NaN values are not allowed and will cause a panic.

## Overview

| Fitness Function Type | Purpose | Use Case | Complexity |
|----------------------|---------|----------|------------|
| [Simple Functions](#simple-fitness-functions) | Basic optimization | Mathematical problems, benchmarks | Low |
| [Composite Functions](#composite-fitness-functions) | Multi-objective combination | Balancing multiple goals | Medium |
| [Novelty Search](#novelty-search) | Behavioral diversity | Exploration, avoiding local optima | High |
| [Problem-Based](#problem-based-fitness) | Encapsulated problems | Complex domains, reusability | Variable |

___

## Simple Fitness Functions

Simple fitness functions are the most common type - they take a phenotype and return a single score value. These can be any function that evaluates how well an individual performs.

---

### Mathematical Functions

Mathematical functions are commonly used for benchmarking and testing optimization algorithms.

=== ":fontawesome-brands-python: Python"

    ```python
    import radiate as rd
    import math

    def rastrigin_function(x: List[float]) -> float:
        A = 10.0
        n = len(x)
        result = A * n
        
        for i in range(n):
            result += x[i]**2 - A * math.cos(2 * math.pi * x[i])
        
        return result

    engine = rd.GeneticEngine(
        codec=rd.FloatCodec.vector(10, (-5.12, 5.12)),
        fitness_func=rastrigin_function,
        objectives="min"
    )
    ```

=== ":fontawesome-brands-rust: Rust"

    ```rust
    use radiate::*;

    fn rastrigin_function(genotype: Vec<f32>) -> f32 {
        const A: f32 = 10.0;
        let n = genotype.len() as f32;
        let mut result = A * n;
        
        for &x in &genotype {
            result += x.powi(2) - A * (2.0 * std::f32::consts::PI * x).cos();
        }
        
        result
    }

    let engine = GeneticEngine::builder()
        .codec(FloatCodec::vector(10, -5.12..5.12))
        .minimizing()
        .fitness_fn(rastrigin_function)
        .build();
    ```

---

### Combinatorial Problems

For problems like TSP, N-Queens, or Knapsack, the fitness function evaluates constraint satisfaction and optimization.

=== ":fontawesome-brands-python: Python"

    ```python
    def tsp_fitness(tour: List[int]) -> float:
        total_distance = 0.0
        for i in range(len(tour)):
            j = (i + 1) % len(tour)
            total_distance += distance_matrix[tour[i]][tour[j]]
        return total_distance

    engine = rd.GeneticEngine(
        codec=rd.PermutationCodec(cities),
        fitness_func=tsp_fitness,
        objectives="min"
    )
    ```

=== ":fontawesome-brands-rust: Rust"

    ```rust
    fn tsp_fitness(genotype: Vec<usize>) -> f32 {
        let mut total_distance = 0.0;
        for i in 0..genotype.len() {
            let j = (i + 1) % genotype.len();
            total_distance += distance_matrix[genotype[i]][genotype[j]];
        }
        total_distance
    }
    ```

---

### Machine Learning Problems

For regression, classification, or neural network evolution, fitness functions evaluate prediction accuracy.

=== ":fontawesome-brands-python: Python"

    ```python
    def regression_fitness(tree: Tree) -> float:
        total_error = 0.0
        for input_data, target in training_data:
            prediction = tree.evaluate(input_data)
            error = (prediction - target) ** 2
            total_error += error
        return total_error / len(training_data)  # MSE

    engine = rd.GeneticEngine(
        codec=rd.TreeCodec.single(3, operations),
        fitness_func=regression_fitness,
        objectives="min"
    )
    ```

=== ":fontawesome-brands-rust: Rust"

    ```rust
    fn regression_fitness(tree: &Tree<Op<f32>>) -> f32 {
        let mut total_error = 0.0;
        for (input, target) in &training_data {
            let prediction = tree.eval(input);
            let error = (prediction - target).powi(2);
            total_error += error;
        }
        total_error / training_data.len() as f32  // MSE
    }
    ```

---

## Composite Fitness Functions

Composite fitness functions allow you to combine multiple objectives into a single weighted fitness score. This is useful when you have multiple goals that need to be balanced.

---

### Weighted Combination

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

---

### Equal Weight Combination

You can also add fitness functions with equal weights using `add_fitness_fn()`.

<!-- ```python
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

---

### How Novelty Search Works

Novelty search works by:

1. **Behavioral Descriptors**: Each individual is described by a behavioral descriptor (e.g., output patterns, feature values)
2. **Archive**: Novel solutions are stored in an archive
3. **Distance Calculation**: Novelty is measured as the average distance to the k-nearest neighbors in the archive
4. **Threshold**: Solutions with novelty above a threshold are added to the archive

---

### Basic Novelty Search

<!-- 
    ```python
    import radiate as rd

    # Define a behavioral descriptor
    class BehaviorDescriptor:
        def description(self, individual: Model) -> List[float]:
            # Return behavioral characteristics (e.g., outputs on test cases)
            return individual.get_behavior_vector()
        
        def distance(self, a: List[float], b: List[float]) -> float:
            # Calculate Euclidean distance between behaviors
            return sum((x - y) ** 2 for x, y in zip(a, b)) ** 0.5

    # Create novelty search fitness function
    novelty_fitness = rd.NoveltySearch(
        behavior=BehaviorDescriptor(),
        k=10,           # Number of nearest neighbors to consider
        threshold=0.1   # Novelty threshold for archive addition
    )

    engine = rd.GeneticEngine(
        codec=rd.ModelCodec(),
        fitness_func=novelty_fitness,
        objectives="max"  # Higher novelty is better
    )
    ``` -->

=== ":fontawesome-brands-python: Python"

    !!! warning ":construction: Under Construction :construction:"
        This function is currently under construction and not yet available in the Python API.

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
    );

    let engine = GeneticEngine::builder()
        .codec(model_codec)
        .maximizing()
        .fitness_fn(novelty_fitness)
        .build();
    ```

---

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

---

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
3. **Novelty Search**: Use when you need to explore diverse solutions or avoid local optima
4. **Problem-Based**: Use for complex domains where you need full control over the problem definition

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
4. **Problem Encapsulation**: Implement the Problem trait for complex, reusable problem definitions