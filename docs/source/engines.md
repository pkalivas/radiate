

The `GeneticEngine` is the core component. Once built, it manages the entire evolutionary process, including population management, fitness evaluation, and genetic operations. The engine itself is essentially a large iterator that produces `Epoch` objects representing each generation.

---

## Epochs

Each epoch represents a single generation in the evolutionary process. An epoch contains information related not only the current generation, but also the engine's state at that point in time. This is the primary output of the engine, and it can be used to track progress, visualize results, or make decisions based on the evolutionary process. 

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
    result = engine.run(rd.GenerationsLimit(100))

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
    // `GeneticEngine<FloatChromosome, f32>`
    //
    // Where the `epoch` is `Generation<FloatChromosome, f32>`
    let mut engine = GeneticEngine::builder()
        .codec(FloatCodec::scalar(0.0..1.0)) 
        .fitness_fn(|genotype: f32| my_fitness_fn(genotype)) // Return a single fitness score
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

### Multi-Objective Epoch

When the engine is configured for multi-objective optimization, the engine `Generation` will have a `ParetoFront` attached to it. The only difference between the single-objective and multi-objective is the availablity of the `ParetoFront` and the `fitness` value. The `fitness` value will be a list of scores, one for each objective being optimized.

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
    result = engine.run(rd.GenerationsLimit(100))

    # Everything in the multi-objective epoch is the same as the single-objective epoch, except for the value:
    # This will be a list of objects representing your pareto front as such:
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
    // `GeneticEngine<FloatChromosome, f32>`
    //
    // Where the `epoch` is `Generation<FloatChromosome, f32>`
    let mut engine = GeneticEngine::builder()
        .codec(FloatCodec::scalar(0.0..1.0)) 
        .multi_objective(vec![Objective::Min, Objective::Max]) // Specify multi-objective optimization
        .fitness_fn(|genotype: f32| my_fitness_fn(genotype)) // Return a multi-objective fitness score
        // ... other parameters ...
        .build();

    // Run the engine for 100 generations - the result will be a `MultiObjectiveGeneration<FloatChromosome>`
    let result = engine.run(|generation: Generation<FloatChromosome, 32>| {
        generation.index() >= 100
    });

    // -- or using the engine's iterator --
    let result = engine.iter().take(100).last().unwrap();

    // Everything in this generation is the same as the single-objective epoch, except that 
    // the function call to `front()` will return a `ParetoFront` object.:
    // This will be of type `Front<Phenotype<FloatChromosome>>`
    let front: Option<&Front<Phenotype<FloatChromosome>>> = result.front();

    // Get the members of the Pareto front:
    let individuals: &[Arc<Phenotype<FloatChromosome>>] = front.values();
    ```

---

## Iterator API

The `GeneticEngine` is an inherently iterable concept, as such we can treat the engine as an iterato. Because of this we can use it in a `for` loop or with iterator methods like `map`, `filter`, etc. We can also extend the iterator with custom methods to provide additional functionality, such as running until a certain fitness (score) is reached, time limit, or convergence. These custom methods are essentially sytactic sugar for 'take_until' or 'skip_while' style iterators.

!!! warning "Stopping Condition"

    The engine's iterator is an 'infinite iterator', meaning it will continue to produce epochs until a stopping condition, a `break` or a `return` is met. So, unless you want to run the engine indefinitely, you should always use a method like `take`, `until`, or `last` to limit the number of epochs produced.
    

=== ":fontawesome-brands-python: Python"

    ```python
    import radiate as rd

    # Create an engine
    engine = rd.GeneticEngine(
        codec=rd.FloatCodec.scalar(0.0, 1.0), 
        fitness_funn=my_fitness_fn,  # Some fitness function
        # ... other parameters ...
    )

    # use a simple for loop to iterate through 100 generations
    for epoch in engine:
        if epoch.index() >= 100:
            break
        print(f"Generation {epoch.index()}: Score = {epoch.score()}")
    ```

=== ":fontawesome-brands-rust: Rust"

    ```rust
    use radiate::*;
    use std::time::Duration;

    // Create an engine
    let mut engine = GeneticEngine::builder()
        .codec(FloatCodec::scalar(0.0..1.0)) 
        .fitness_fn(|genotype: f32| my_fitness_fn(genotype))
        // ... other parameters ...
        .build();

    // 1.) use a simple for loop to iterate through 100 generations
    for epoch in engine.iter().take(100) {
        println!("Generation {}: Score = {}", epoch.index(), epoch.score().as_f32());
    }

    // 2.) use the iterator's custom methods to run until a score target is reached
    let target_score = 0.01;
    let result = engine.iter().until_score(target_score).take(1).last().unwrap();

    // 3.) run until a time limit is reached
    let time_limit = Duration::from_secs(60);
    let result = engine.iter().until_duration(time_limit).take(1).last().unwrap();

    // 4.) run until convergence
    let window = 50;
    let epsilon = 0.01; // how close the scores must be over the window to consider convergence
    let result = engine.iter().until_convergence(window, epsilon).take(1).last().unwrap();
    ```
---

## Problem API

For certain optimization problems, it is useful to have a more structured way to define a `problem`. For instance, it may be useful to hold stateful information within a fitness function, store data in a more unified way, or evaluate a `Genotype<C>` directly without decoding. The `problem` interface provides a way to do just that. Under the hood of the `GeneticEngine`, the builder constructs a `problem` object that holds the `codec` and fitness function. Because of this, when using the `problem` API, we don't need a `codec` or a fitness function - the `problem` will take care of that for us. 

=== ":fontawesome-brands-python: Python"

    !!! warning ":construction: Under Construction :construction:"

        The problem API in Python is still under construction and is not yet available.

=== ":fontawesome-brands-rust: Rust"

    ```rust
    use radiate::*;

    // Define a problem struct that holds stateful information
    struct MyFloatProblem {
        num_genes: usize,
        value_range: Range<f32>,
    }

    impl Problem<FloatChromosome, Vec<f32>> for MyFloatProblem {
        fn encode(&self) -> Genotype<FloatChromosome> {
            Genotype::from(FloatChromosome::from((self.num_genes, self.value_range.clone())))
        }
        
        fn decode(&self, genotype: &Genotype<FloatChromosome>) -> Vec<f32> {
            genotype.genes().iter().map(|gene| gene.value()).collect()
        }

        fn eval(&self, genotype: &Genotype<FloatChromosome>) -> Score {
            // Evaluate the genotype directly without decoding
            my_fitness_fn(&genotype)
        }
    }

    // The `Problem<C, T>` trait requires `Send` and `Sync` implementations
    unsafe impl Send for MyFloatProblem {}
    unsafe impl Sync for MyFloatProblem {}

    // Create an engine with the problem
    let mut engine = GeneticEngine::builder()
        .problem(MyProblem { num_genes: 10, value_range: 0.0..1.0 })
        .build();

    // Run the engine
    let result = engine.run(|epoch| epoch.index() >= 100);
    ```

---

## Metrics

The `MetricSet`, included in the engine's epoch, provides a number of built-in metrics that can be used to evaluate the performance of the `GeneticEngine`. These metrics can be used to monitor the progress of the engine, compare different runs, and tune hyperparameters. During evolution, the engine collects various metrics from it's different components as well as the overall performance of the engine. 

A metric is defined as:

1. `Value` - Represents a single value metric with a name and a `Statistic`.
2. `Time` - Represents a time metric with a name and a `TimeStatistic`.
3. `Distribution` - Represents a distribution metric with a name and a `Distribution`.

### Statistic 

The `Statistic` exposes a number of different statistical measures that can be used to summarize the data, such as, `last_value`, `count`, `min`, `max`, `mean`, `sum`, `variance`, `std_dev`, `skewness`, and `kurtosis`. 

### TimeStatistic

Similarly, the `TimeStatistic` exposes the same measures, however the data is assumed to be time-based. As such, the results are expressed as a `Duration::from_secs_f32(value)`.

### Distribution

The `Distribution` metric is used to represent a distribution of values. The distribution is stored as a `Vec<f32>` and produces the same statistical measures as the `Statistic` and `TimeStatistic` with the exception of `last_value` which is changed to `last_sequence`.

The default metrics collected by the engine are:


| Name                | Type          | Description                                                                 |
|---------------------|---------------|-----------------------------------------------------------------------------|
| `time`              | TimeStatistic | The time taken for the evolution process.                                   |
| `scores`            | Statistic     | The scores (fitness) of all the individuals evolved throughout the evolution process. |
| `age`               | Statistic     | The age of all the individuals in the `Ecosystem`. |
| `replace_age`      | Statistic     | The number of individuals replaced based on age. |
| `replace_invalid`  | Statistic     | The number of individuals replaced based on invalid structure (e.g. Bounds) |
| `genome_size`      | Distribution   | The size of each genome over the evolution process. This is usually static and doesn't change. |
| `front`            | Statistic   | The number of members added to the Pareto front throughout the evolution process. |
| `unique_members`   | Statistic     | The number of unique members in the `Ecosystem`. |
| `unique_scores`    | Statistic     | The number of unique scores in the `Ecosystem`. |
| `diversity_ratio`  | Statistic     | The ratio of unique scores to the size of the `Ecosystem`. |
| `score_volatility` | Statistic     | The volatility of the scores in the `Ecosystem`. This is calculated as the standard deviation of the scores / mean. |
| `species_count`    | Statistic     | The number of `species` in the 'Ecosystem`. |
| `species_removed`  | Statistic     | The number of `species` removed based on stagnation. |
| `species_distance` | Distribution | The distance between `species` in the `Ecosystem`. |
| `species_created`  | Statistic     | The number of `species` created in the `Ecosystem`. |
| `species_died`     | Statistic     | The number of `species` that have died in the `Ecosystem`. |
| `species_age`      | Statistic     | The age of all the `species` in the `Ecosystem`. |

Along with the default metrics, each component will also collect operation metrics (statistics and time statistics) for the operations it performs. For example, each `Alterer` and `Selector` will collect metrics and be identified by their name. Its also important to note that `species` level metrics will only be collected if the engine is configured to use species-based diversity.

These can be accessed through the `metrics()` method of the epoch, which returns a `MetricSet`. 

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

    # Get the metrics of the engine
    metrics = result.metrics()  # MetricSet object

    # Access specific metrics
    time_taken = metrics["time"]['time_sum'] # Total time taken for the evolution process
    scores = metrics["scores"] # dict of score statistics

    mean_score = scores['value_mean']  # Mean score of all individuals
    all_last_generation_scores = scores['sequence_last']  # Last generation scores
    ```

=== ":fontawesome-brands-rust: Rust"

    ```rust
    // --- set up the engine ---

    let result = engine.run(|ctx| {
        // get the scroe metric from the generation context
        let temp = ctx.metrics().get("scores").unwrap();
        // get the standard deviation of the score distribution
        let std = temp.value_std_dev();
        
        std < 0.01 // Example condition to stop the engine
    });

    // Access the metrics from the result
    let metrics: MetricSet = result.metrics();
    ```

---

## Tips

* Use appropriate population sizes (100-500 for most problems)
* Enable parallel execution for expensive fitness functions
* Use efficient selection strategies for large populations
* Consider species-based diversity for complex landscapes

