# Objectives

___
Objectives define the direction of optimization for your genetic algorithm. They determine whether the algorithm should minimize or maximize the fitness function, and support both single-objective and multi-objective optimization problems.

The choice of objective is fundamental to the genetic algorithm's behavior, as it directly influences how individuals are ranked, selected, and evolved. Understanding how to properly configure objectives is essential for achieving optimal results in your optimization problems.

!!! note "**The default objective is to maximize a single objective**."

    This means that if you do not specify an objective, the algorithm will assume you want to maximize a single fitness function. If you want to minimize or use multi-objective optimization, you must explicitly configure it - see below.

## Overview

| Objective Type | Ex. Use Cases | Complexity | Performance |
|----------------|----------|------------|-------------|
| [Single Minimize](#minimization) | Error/loss functions, cost optimization | Low | High |
| [Single Maximize](#maximization) | Profit/revenue, performance metrics | Low | High |
| [Multi-Objective](#multi-objective-optimization) | Conflicting objectives, trade-off analysis | High | Medium |

___

## Single-Objective Optimization

Single-objective optimization focuses on optimizing one specific goal. This is by far the most common use case for genetic algorithms. 
When using a single objective, you must return only a single value from your fitness function, which represents the fitness score for that individual.

---

### Minimization

> **Purpose**: Find the minimum value of the fitness function

The `minimizing()` method configures the genetic algorithm to find the minimum value of your fitness function. This is commonly used for error functions, cost optimization, and any scenario where you want to reduce a metric.

=== ":fontawesome-brands-python: Python"

    ```python
    --8<-- "python/objectives.py:minimizing"
    ```

=== ":fontawesome-brands-rust: Rust"

    ```rust
    use radiate::*;

    let engine = GeneticEngine::builder()
        .codec(FloatCodec::vector(10, 0.0..1.0))  // Example codec
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

This is the default option for the `GeneticEngine`, so you don't really need to explicitly set this, but the functionality is provided for clarity and explicitness. The `maximizing()` method configures the genetic algorithm to find the maximum value of your fitness function. This is commonly used for profit optimization, performance metrics, and any scenario where you want to increase a metric.

=== ":fontawesome-brands-python: Python"

    ```python
    --8<-- "python/objectives.py:maximizing"
    ```

=== ":fontawesome-brands-rust: Rust"

    ```rust
    use radiate::*;

    let engine = GeneticEngine::builder()
        .codec(FloatCodec::vector(10, 0.0..1.0))  // Example codec
        .maximizing()  // Configure for maximization
        .fitness_fn(|genotype| {
            // Return a value to maximize
            genotype.iter().sum::<f32>()
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

Multi-objective optimization allows you to optimize multiple conflicting objectives simultaneously. Instead of finding a single "best" solution, you find a set of Pareto-optimal solutions that represent different trade-offs between objectives. You'll notice in the examples below that the fitness function returns a list of values, each representing a different objective. The number of objectives should match the number of directions specified.

> **Purpose**: Optimize multiple conflicting objectives simultaneously

With multiple objectives there's rarely a single "best" solution — improving one objective usually means giving ground on another. The key idea is **Pareto dominance**: solution *A* dominates solution *B* if it is at least as good as *B* on **every** objective and strictly better on **at least one**. The solutions that *no* other solution dominates form the **Pareto front** — the set of best available trade-offs. Rather than collapsing everything down to one winner, multi-objective optimization evolves and returns this whole front, leaving the final trade-off choice to you.

---

### Setting Up MO Problems

Use `multi_objective()` with a list of optimization directions to configure multi-objective optimization. To bound how many solutions the engine keeps on the Pareto front, set the **front-size range** — `front_range(min, max)` in Python, `front_size(min..max)` in Rust. It defaults to `800..900`.

=== ":fontawesome-brands-python: Python"

    ```python
    --8<-- "python/objectives.py:multi_objective"
    ```

=== ":fontawesome-brands-rust: Rust"

    ```rust
    use radiate::*;

    let engine = GeneticEngine::builder()
        .codec(FloatCodec::vector(10, 0.0..1.0))  // Example codec
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

---

### Pareto Front Management

The **front-size range** controls how large the Pareto front is allowed to grow. When the front fills up (reaches the upper bound), the engine truncates it back down to the lower bound, dropping solutions by Pareto dominance and then crowding distance — so it keeps the most diverse, least-dominated set.

---

### MO Selectors

Because of the complexity of multi-objective problems, specialized selectors are available which are optimized for handling Pareto dominance and diversity:

- `NSGA2Selector`: Implements the NSGA-II algorithm for non-dominated sorting and crowding distance
- `NSGA3Selector`: Implements the NSGA-III algorithm for many-objective optimization
- `TournamentNSGA2Selector`: Uses tournament selection with Pareto dominance

Although, any selector can be used, these are optimized for multi-objective problems. The other selectors will work by computing a 'weight' based off of the pareto dominance and crowding distance, but they are not backed by any specific multi-objective algorithm.

=== ":fontawesome-brands-python: Python"

    ```python
    --8<-- "python/objectives.py:multi_objective_selectors"
    ```

=== ":fontawesome-brands-rust: Rust"

    ```rust
    use radiate::*;

    let engine = GeneticEngine::builder()
        .codec(FloatCodec::vector(10, 0.0..1.0))  // Example codec
        .multi_objective(vec![Optimize::Minimize, Optimize::Maximize])
        .offspring_selector(TournamentNSGA2Selector::new(3))    // Tournament selection with Pareto dominance
        .survivor_selector(NSGA3Selector::new(12))              // NSGA-III with 12 reference directions
        .front_size(800..900)  // Pareto front size range
        .fitness_fn(|genotype| {
            vec![
                objective1(genotype),  // Minimize this
                objective2(genotype),  // Maximize this
            ]
        })
        .build();
    ```
    
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

1. **Problem**: Algorithm converges to poor solutions
    - Solution: Check objective direction (minimize vs maximize)

2. **Problem**: Multi-objective front is too small
    - Solution: Increase front size range or adjust selection pressure
