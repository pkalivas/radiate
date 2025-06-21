# Objectives

___
Objectives define the direction of optimization for your genetic algorithm. They determine whether the algorithm should minimize or maximize the fitness function, and support both single-objective and multi-objective optimization problems.

The choice of objective is fundamental to the genetic algorithm's behavior, as it directly influences how individuals are ranked, selected, and evolved. Understanding how to properly configure objectives is essential for achieving optimal results in your optimization problems.

## Overview

| Objective Type | Use Case | Complexity | Performance |
|----------------|----------|------------|-------------|
| [Single Minimize](#single-objective-minimization) | Error/loss functions, cost optimization | Low | High |
| [Single Maximize](#single-objective-maximization) | Profit/revenue, performance metrics | Low | High |
| [Multi-Objective](#multi-objective-optimization) | Conflicting objectives, trade-off analysis | High | Medium |

___

## Single-Objective Optimization

Single-objective optimization focuses on optimizing one specific goal. This is the most common use case and provides the best performance characteristics.

---

### Minimization

> **Purpose**: Find the minimum value of the fitness function

The `minimizing()` method configures the genetic algorithm to find the minimum value of your fitness function. This is commonly used for error functions, cost optimization, and any scenario where you want to reduce a metric.

=== ":fontawesome-brands-python: Python"

    ```python
    import radiate as rd

    engine = rd.GeneticEngine(
        # ... codec and other parameters ...
        fitness_func=lambda x: sum(x),  # Sum to minimize
        objectives="min" # Configure for minimization
        # ... other parameters ...
    )
    
    # Or using builder pattern
    engine = rd.GeneticEngine(codec=codec, fitness_func=lambda x: sum(x))
    engine.minimizing()
    ```

=== ":fontawesome-brands-rust: Rust"

    ```rust
    use radiate::*;

    let engine = GeneticEngine::builder()
        // ... codec and other parameters ...
        .minimizing() // Configure for minimization
        .fitness_fn(|genotype| {
            // Return a value to minimize
            genotype.iter().sum::<f32>()
        })
        // ... other parameters ...
        .build();
    ```

**Common Applications:**

- **Error Functions**: Minimize prediction error in machine learning
- **Cost Optimization**: Minimize production costs, travel distance
- **Constraint Violations**: Minimize penalty for constraint violations
- **Loss Functions**: Minimize training loss in neural networks

---

### Maximization

> **Purpose**: Find the maximum value of the fitness function

The `maximizing()` method configures the genetic algorithm to find the maximum value of your fitness function. This is commonly used for profit optimization, performance metrics, and any scenario where you want to increase a metric.

=== ":fontawesome-brands-python: Python"

    ```python
    import radiate as rd

    engine = rd.GeneticEngine(
        # ... codec and other parameters ...
        fitness_func=lambda x: -sum(x),  # Negative sum to maximize
        objectives="max"
        # ... other parameters ...
    )
    
    # Or using builder pattern
    engine = rd.GeneticEngine(codec=codec, fitness_func=lambda x: -sum(x))
    engine.maximizing()
    ```

=== ":fontawesome-brands-rust: Rust"

    ```rust
    use radiate::*;

    let engine = GeneticEngine::builder()
        // ... codec and other parameters ...
        .maximizing()  // Configure for maximization
        .fitness_fn(|genotype| {
            // Return a value to maximize
            genotype.iter().product::<f32>()
        })
        // ... other parameters ...
        .build();
    ```

**Common Applications:**

- **Profit Optimization**: Maximize revenue, return on investment
- **Performance Metrics**: Maximize accuracy, precision, recall
- **Quality Scores**: Maximize product quality, user satisfaction
- **Resource Utilization**: Maximize efficiency, throughput

---

## Multi-Objective Optimization

Multi-objective optimization allows you to optimize multiple conflicting objectives simultaneously. Instead of finding a single "best" solution, you find a set of Pareto-optimal solutions that represent different trade-offs between objectives.

> **Purpose**: Optimize multiple conflicting objectives simultaneously

---

### Setting Up MO Problems

Use `multi_objective()` with a list of optimization directions to configure multi-objective optimization. To control the size of the Pareto front, use `front_range()` to specify the minimum and maximum number of solutions to maintain as seen below:

=== ":fontawesome-brands-python: Python"

    ```python
    import radiate as rd

    engine = rd.GeneticEngine(
        # ... codec and other parameters ...
        fitness_func=lambda x: [cost(x), quality(x)],  # Return list of objectives
        objectives=["min", "max"]  # Minimize cost, maximize quality
        front_range=(800, 900)  # Pareto front size range
        # ... other parameters ...
    )
    
    # Or using builder pattern
    engine = rd.GeneticEngine(codec=codec, fitness_func=lambda x: [cost(x), quality(x)])
    engine.multi_objective(["min", "max"], front_range=(800, 900))
    ```

=== ":fontawesome-brands-rust: Rust"

    ```rust
    use radiate::*;

    let engine = GeneticEngine::builder()
        // ... codec and other parameters ...
        .multi_objective(vec![Optimize::Minimize, Optimize::Maximize])
        .front_size(800..900)  // Pareto front size range
        .fitness_fn(|genotype| {
            // Return a vector of fitness values
            vec![
                objective1(genotype),  // Minimize this
                objective2(genotype),  // Maximize this
            ]
        })
        // ... other parameters ...
        .build();
    ```

### Pareto Front Management

The `front_size()` parameter controls the size of the Pareto front. When the pareto front is full (reaches the upper bound), the algorithm will truncate down to the lower bound by removing solutions based on Pareto dominance and crowding distance.

### Multi-Objective Selectors

Because of the complexity of multi-objective problems, specialized selectors are available which are optimized for handling Pareto dominance and diversity:

- `NSGA2Selector`: Implements the NSGA-II algorithm for non-dominated sorting and crowding distance
- `TournamentNSGA2Selector`: Uses tournament selection with Pareto dominance

Although, any selector can be used, these are optimized for multi-objective problems.

=== ":fontawesome-brands-python: Python"

    ```python
    import radiate as rd

    engine = rd.GeneticEngine(
        # ... codec and other parameters ...
        fitness_func=lambda x: [obj1(x), obj2(x)],
        front_range=(800, 900),  # Pareto front size range
        objectives=["min", "max"]
    )
    engine.survivor_selector(rd.NSGA2Selector())                # NSGA-II for Pareto ranking
    engine.offspring_selector(rd.TournamentNSGA2Selector(k=3))  # Tournament selection with Pareto dominance
    ```

=== ":fontawesome-brands-rust: Rust"

    ```rust
    use radiate::*;

    let engine = GeneticEngine::builder()
        // ... codec and other parameters ...
        .multi_objective(vec![Optimize::Minimize, Optimize::Maximize])
        .survivor_selector(NSGA2Selector::new())                // NSGA-II for Pareto ranking
        .offspring_selector(TournamentNSGA2Selector::new(3))    // Tournament selection with Pareto dominance
        .front_size(800..900)  // Pareto front size range
        .fitness_fn(|genotype| {
            vec![
                objective1(genotype),  // Minimize this
                objective2(genotype),  // Maximize this
            ]
        })
        .build();
    ```

<!-- ## Examples -->

---

## Best Practices

### Choosing Objectives

1. **Single-objective**: Use when you have a clear, single goal
2. **Multi-objective**: Use when objectives conflict or you need trade-off analysis
3. **Objective count**: Keep multi-objective channels to a manageable number

### Performance Considerations

1. **Single-objective**: Generally faster and more focused
2. **Multi-objective**: More computationally intensive
3. **Front size**: Larger fronts provide better coverage but increase memory usage and computation time
4. **Objective count**: More objectives increase complexity exponentially

---

## Troubleshooting

### Common Issues

**Problem**: Algorithm converges to poor solutions
**Solution**: Check objective direction (minimize vs maximize)

**Problem**: Multi-objective front is too small
**Solution**: Increase front size range or adjust selection pressure

**Problem**: Solutions cluster in one area
**Solution**: Use crowding distance or diversity maintenance

**Problem**: Slow convergence in multi-objective
**Solution**: Use NSGA-II selector and appropriate crossover/mutation rates
