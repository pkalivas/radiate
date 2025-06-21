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
    import radiate as rd

    codec = rd.FloatCodec.vector(10, (0.0, 1.0))  # Example codec

    engine = rd.GeneticEngine(
        codec=codec,
        fitness_func=lambda x: sum(x),  # value to minimize
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
    import radiate as rd

    codec = rd.FloatCodec.vector(10, (0.0, 1.0))  # Example codec

    engine = rd.GeneticEngine(
        codec=codec,
        fitness_func=lambda x: sum(x),  # return a value to maximize
        objectives="max"
        # ... other parameters ...
    )
    
    # Or using builder pattern
    engine = rd.GeneticEngine(codec=codec, fitness_func=lambda x: sum(x))
    engine.maximizing()
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

---

### Setting Up MO Problems

Use `multi_objective()` with a list of optimization directions to configure multi-objective optimization. To control the size of the Pareto front, use `front_range()` to specify the minimum and maximum number of solutions to maintain as seen below:

=== ":fontawesome-brands-python: Python"

    ```python
    import radiate as rd
    
    codec = rd.FloatCodec.vector(10, (0.0, 1.0))  # Example codec

    engine = rd.GeneticEngine(
        codec=codec,
        fitness_func=lambda x: [obj1_fitness_func(x), obj2_fitness_func(x)],  # Return list of objectives
        objectives=["min", "max"]  # Minimize obj1, maximize obj2
        front_range=(800, 900)  # Pareto front size range
        # ... other parameters ...
    )
    
    # Or using builder pattern
    engine = rd.GeneticEngine(codec=codec, fitness_func=lambda x: [obj1_fitness_func(x), obj2_fitness_func(x)])
    engine.multi_objective(["min", "max"], front_range=(800, 900))
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

The `front_size()` parameter controls the size of the Pareto front. When the pareto front is full (reaches the upper bound), the algorithm will truncate down to the lower bound by removing solutions based on Pareto dominance and crowding distance.

---

### MO Selectors

Because of the complexity of multi-objective problems, specialized selectors are available which are optimized for handling Pareto dominance and diversity:

- `NSGA2Selector`: Implements the NSGA-II algorithm for non-dominated sorting and crowding distance
- `TournamentNSGA2Selector`: Uses tournament selection with Pareto dominance

Although, any selector can be used, these are optimized for multi-objective problems. The other selectors will work by computing a 'weight' based off of the pareto dominance and crowding distance, but they are not backed by any specific multi-objective algorithm.

=== ":fontawesome-brands-python: Python"

    ```python
    import radiate as rd

    engine = rd.GeneticEngine(
        codec=rd.FloatCodec.vector(10, (0.0, 1.0)),  # Example codec
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
        .codec(FloatCodec::vector(10, 0.0..1.0))  // Example codec
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

---

### MO Examples

The [DTLZ1](https://pymoo.org/problems/many/dtlz.html) problem is a well-known multiobjective optimization problem that is used to test the performance of multiobjective optimization algorithms. It is a 3-objective problem with 4 variables and is defined as:

$$
\begin{align*}
\text{minimize} \quad & f_1(x) = (1 + g) \cdot x_1 \cdot x_2 \\
\text{minimize} \quad & f_2(x) = (1 + g) \cdot x_1 \cdot (1 - x_2) \\
\text{minimize} \quad & f_3(x) = (1 + g) \cdot (1 - x_1) \\
\text{subject to} \quad & 0 \leq x_i \leq 1 \quad \text{for} \quad i = 1, 2, 3, 4 \\
\text{where} \quad & g = \sum_{i=3}^{4} (x_i - 0.5)^2
\end{align*}
$$

=== ":fontawesome-brands-python: Python"

    ```python
    import radiate as rd

    variables = 4
    objectives = 3
    k = variables - objectives + 1

    def dtlz_1(val):
        g = 0.0
        for i in range(variables - k, variables):
            g += (val[i] - 0.5) ** 2 - math.cos(20.0 * math.pi * (val[i] - 0.5))
        g = 100.0 * (k + g)
        f = [0.0] * objectives
        for i in range(objectives):
            f[i] = 0.5 * (1.0 + g)
            for j in range(objectives - 1 - i):
                f[i] *= val[j]
            if i != 0:
                f[i] *= 1.0 - val[objectives - 1 - i]
        return f


    engine = rd.GeneticEngine(
        codec=rd.FloatCodec.vector(variables, (0.0, 1.0), (-100.0, 100.0)),
        fitness_func=dtlz_1,
        offspring_selector=rd.TournamentSelector(k=5),
        survivor_selector=rd.NSGA2Selector(),
        objectives=["min" for _ in range(objectives)],
        alters=[
            rd.SimulatedBinaryCrossover(1.0, 1.0),
            rd.UniformMutator(0.1)
        ],
    )

    result = engine.run(rd.GenerationsLimit(1000))

    # When running an MO problem, we can get the resulting pareto from from the 
    # engine's epoch result. This is stored in the 'value()' field of the result here:
    front = result.value()

    # The front is a list of individuals, each with a 'fitness' attribute
    # which is a list of fitness values for each objective.
    x = [member["fitness"][0] for member in front]
    y = [member["fitness"][1] for member in front]
    z = [member["fitness"][2] for member in front]
    ```

=== ":fontawesome-brands-rust: Rust"

    ``` rust 
    use radiate::*;

    pub fn dtlz_1(values: &[f32]) -> Vec<f32> {
        let mut g = 0.0;
        for i in VARIABLES - K..VARIABLES {
            g += (values[i] - 0.5).powi(2) - (20.0 * std::f32::consts::PI * (values[i] - 0.5)).cos();
        }

        g = 100.0 * (K as f32 + g);

        let mut f = vec![0.0; OBJECTIVES];
        for i in 0..OBJECTIVES {
            f[i] = 0.5 * (1.0 + g);
            for j in 0..OBJECTIVES - 1 - i {
                f[i] *= values[j];
            }

            if i != 0 {
                f[i] *= 1.0 - values[OBJECTIVES - 1 - i];
            }
        }

        f
    }

    const VARIABLES: usize = 4;
    const OBJECTIVES: usize = 3;
    const K: usize = VARIABLES - OBJECTIVES + 1;

    let codec = FloatCodec::vector(VARIABLES, 0_f32..1_f32).with_bounds(-100.0, 100.0);

    let engine = GeneticEngine::builder()
        .codec(codec)
        .multi_objective(vec![Optimize::Minimize; OBJECTIVES])
        .offspring_selector(TournamentSelector::new(5))
        .survivor_selector(NSGA2Selector::new())
        .alter(alters!(
            SimulatedBinaryCrossover::new(1_f32, 1.0),
            UniformMutator::new(0.1),
        ))
        .fitness_fn(|geno: Vec<f32>| dtlz_1(&geno))
        .build();
    
    let result = engine.run(|ctx| {
        println!("[ {:?} ]", ctx.index);
        ctx.index > 1000
    });

    // When running an MO problem, we can get the resulting pareto from from the 
    // engine's epoch result. This is stored in the 'value()' field of the result here:
    let front = result.value();
    ```

The resulting Pareto front can be visualized using Plotly or matplotlib, as shown below:

<div id="dtlz_1"></div>


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


<script src="https://cdn.plot.ly/plotly-latest.min.js"></script>
<script>
Promise.all([
    fetch("../../assets/dtlz_1.json").then(response => response.json()),
    fetch("../../assets/dtlz_2.json").then(response => response.json())
])
.then(([dtlz1, dtlz2]) => {
    let x1 = [], y1 = [], z1 = [];
    let x2 = [], y2 = [], z2 = [];

    dtlz1.pareto_front.forEach(point => {
        x1.push(point[0]);
        y1.push(point[1]);
        z1.push(point[2]);
    });

    dtlz2.pareto_front.forEach(point => {
        x2.push(point[0]);
        y2.push(point[1]);
        z2.push(point[2]);
    });

    let trace1 = {
        x: x1,
        y: y1,
        z: z1,
        mode: "markers",
        type: "scatter3d",
        name: "DTLZ1",
        marker: { size: 5, color: "blue" }
    };

    let trace2 = {
        x: x2,
        y: y2,
        z: z2,
        mode: "markers",
        type: "scatter3d",
        name: "DTLZ2",
        marker: { size: 5, color: "red" }
    };

    Plotly.newPlot("dtlz_1", [trace1]);
    Plotly.newPlot("dtlz_2", [trace2]);
})
.catch(error => console.error(error));
</script>