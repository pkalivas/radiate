

The `GeneticEngine` is the core component. Once built, it manages the entire evolutionary process, including population management, fitness evaluation, and genetic operations. The engine itself is essentially a large iterator that produces `Epoch` objects representing each generation.

---

## Epochs

Each epoch represents a single generation in the evolutionary process. An epoch contains information related not only the current generation, but also the engine's state at that point in time. This is the primary output of the engine, and it can be used to track progress, visualize results, or make decisions based on the evolutionary process. Because there are two main types of optimization problems the engine can solve (single-objective and multi-objective), the engine produces different types of epochs depending on the objective type.

---

### Single-Objective Epoch

This is the default epoch for the engine - `Generation`. It contains:

- The generation number
- `Ecosystem` information (population, species, etc.)
- Score, which is the fitness of the best individual in the generation
- Value, which is the decoded value of the best individual
- Performance metrics (e.g., time taken)
- The Objective (max or min). The fitness objective being optimized, used for comparison and disicion making during the evolutionary process.

=== ":fontawesome-brands-python: Python"

    ```python
    import radiate as rd

    # Create an engine
    engine = rd.GeneticEngine(
        codec=rd.FloatCodec.scalar(0.0, 1.0), 
        fitness_fn=my_fitness_fn,  # Single objective fitness function
        # ... other parameters ...
    )

    # Run the engine for 100 generations
    result = engine.run(rd.GeneratinsLimit(100))

    # Get the best individual's decoded value 
    value = result.value() # float 

    # Get the score (fitness) of the best individual or epoch score
    score = result.score()  # List[float] - note that this is a list. 
    # In this scenario, the engine is configured for single-objective optimization,
    # so the list will contain a single value.

    # Get the population of the engine's ecosystem
    population = result.population()  # Population object

    # Get the index of the epoch (number of generations)
    index = result.index()  # int

    # Get the metrics of the engine
    metrics = result.metrics()  # MetricSet object
    ```

=== ":fontawesome-brands-rust: Rust"

    ```rust
    use radiate::*;

    // Create an engine of type:
    // `GeneticEngine<FloatChromosome, f32, Generation<FloatChromosome, f32>>`
    //
    // Where the `epoch` is `Generation<FloatChromosome, f32>`
    let mut engine = GeneticEngine::builder()
        .codec(FloatCodec::scalar(0.0..1.0)) 
        .fitness_fn(|genotype: Vec<f32>| my_fitness_fn(genotype)) // Return a single fitness score
        // ... other parameters ...
        .build();

    // Run the engine for 100 generations - the result will be a `Generation<FloatChromosome, f32>`
    let result = engine.run(|generation: Generation<FloatChromosome, f32>| {
        generation.index() >= 100
    });

    // -- or using the engine's iterator --
    let result = engine.iter().take(100).last().unwrap();

    // Get the best individual's decoded value and fitness score:
    let best_value: f32 = result.value();

    // Get the score (fitness) of the best individual (or epoch score):
    let best_score: Score = result.score();

    // Get the index of the epoch (number of generations):
    let index: usize = result.index();

    // Get the ecosystem level information:
    let ecosystem: Ecosystem<FloatChromosome> = result.ecosystem();
    let population: Population<FloatChromosome> = ecosystem.population();
    let species: Option<&[Species<FloatChromosome>]> = ecosystem.species();

    // Get performance metrics:
    let metrics: MetricSet = result.metrics();

    // Get evolution duration (also available in metrics):
    let time: Duration = result.time();
    ```

---

### Multi-Objective Epoch

When the engine is configured for multi-objective optimization, the engine produces `MultiObjectiveGeneration` objects. When building the engine, once you specify multi-objective optimization or a front_size, the engine will change types from `Generation` to `MultiObjectiveGeneration`. This epoch contains:

- The generation number
- `Ecosystem` information (population, species, etc.)
- Score, which is the fitness of the best individual in the generation
- Value, which is a pareto front of the indivuals the engine has found so far. 
    - **This is the main difference from the single-objective epoch**. Instead of a single value, it contains a collection of values representing the Pareto front.
- Performance metrics (e.g., time taken)
- The Objective (max or min). The fitness objective being optimized, used for comparison and disicion making during the evolutionary process.

=== ":fontawesome-brands-python: Python"

    ```python
    import radiate as rd

    # Create an engine
    engine = rd.GeneticEngine(
        codec=rd.FloatCodec.scalar(0.0, 1.0), 
        fitness_fn=my_fitness_fn,  # Multi-objective fitness function
        objective=['min', 'max', ...],  # Specify multi-objective optimization
        # ... other parameters ...
    )

    # Run the engine for 100 generations
    result = engine.run(rd.GeneratinsLimit(100))

    # Everything in the multi-objective epoch is the same as the single-objective epoch, except for the value:
    # This will be a list of objects as such:
    # [
    #     {'genotype': [Float], 'fitness': [obj1_fit, obj2_fit, ...]},
    #     {'genotype': [Float], 'fitness': [obj1_fit, obj2_fit, ...]},
    #     ...
    # ]
    value = result.value()  
    ```

=== ":fontawesome-brands-rust: Rust"

    ```rust
    use radiate::*;

    // Create an engine of type:
    // `GeneticEngine<FloatChromosome, f32, MultiObjectiveGeneration<FloatChromosome>>`
    //
    // Where the `epoch` is `MultiObjectiveGeneration<FloatChromosome>`
    let mut engine = GeneticEngine::builder()
        .codec(FloatCodec::scalar(0.0..1.0)) 
        .multi_objective(vec![Objective::Min, Objective::Max]) // Specify multi-objective optimization
        .fitness_fn(|genotype: Vec<f32>| my_fitness_fn(genotype)) // Return a multi-objective fitness score
        // ... other parameters ...
        .build();

    // Run the engine for 100 generations - the result will be a `MultiObjectiveGeneration<FloatChromosome>`
    let result = engine.run(|generation: MultiObjectiveGeneration<FloatChromosome>| {
        generation.index() >= 100
    });

    // -- or using the engine's iterator --
    let result = engine.iter().take(100).last().unwrap();

    // Everything in the multi-objective epoch is the same as the single-objective epoch, except for the value:
    // This will be of type `Front<Phenotype<FloatChromosome>>`
    let front: Front<Phenotype<FloatChromosome>> = result.value();

    // Get the members of the Pareto front:
    let individuals: &[Arc<Phenotype<FloatChromosome>>] = front.values();
    ```

---

<!-- ## Execution Models

### Iterator Pattern

The engine implements the `Iterator` trait, allowing fine-grained control:

```rust
let mut engine = GeneticEngine::builder()
    .codec(codec)
    .fitness_fn(fitness_fn)
    .build();

// Manual iteration
for epoch in engine.iter().take(100) {
    println!("Generation {}: Score = {}", epoch.index(), epoch.score().as_f32());
}
```

### Run Until Condition

Execute until a stopping condition is met:

```rust
let result = engine.run(|epoch| {
    epoch.score().as_f32() < 0.01 || epoch.index() >= 1000
});
```

---

### Convenience Iterators

The engine provides specialized iterators for common patterns:

```rust
// Run until score target
engine.iter().until_score_equal(target);

// Run for time limit
engine.iter().until_time_limit(Duration::from_secs(60));

// Run until convergence
engine.iter().until_convergence(50);
``` -->

<!-- --- -->
<!-- 
## Threading and Parallelism

The engine supports different execution strategies:

* **Serial Execution**: Single-threaded execution (default)
* **Worker Pool**: Multi-threaded execution with configurable thread count
* **Custom Executors**: User-defined parallel execution strategies

--- -->

#### Engine Tips

* Use appropriate population sizes (100-500 for most problems)
* Enable parallel execution for expensive fitness functions
* Use efficient selection strategies for large populations
* Consider species-based diversity for complex landscapes
* Monitor memory usage with large populations or complex chromosomes

