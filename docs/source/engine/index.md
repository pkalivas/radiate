# Genetic Engine

The `GeneticEngine` is the core component. Once built, it manages the entire evolutionary process, including population management, fitness evaluation, and genetic operations. The engine itself is essentially a large iterator that produces `Generation` objects representing each generation.

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
- The Objective (max or min). The fitness objective being optimized, used for comparison and decision making during the evolutionary process.

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

    # Get the objective of the engine
    objective = result.objective()  # list[str] | str (list[str] if multi-objective) - "min" or "max"
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
    // Note - the result needs to be 'mut' to access these methods as 
    // they may require mutable access internally - caching, etc.
    let ecosystem: Ecosystem<FloatChromosome> = result.ecosystem();
    let population: Population<FloatChromosome> = ecosystem.population();
    let species: Option<&[Species<FloatChromosome>]> = ecosystem.species();

    // Get performance metrics:
    let metrics: MetricSet = result.metrics();

    // Get evolution duration (also available in metrics):
    let time: Duration = result.time();

    // Get the objective of the engine
    let objective: &Objective = result.objective(); 
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

    # Everything in the multi-objective epoch is the same as the single-objective epoch, except for the value. 
    # The function call to `front()` will return a `ParetoFront` object while `value()` will return None.:
    front = result.front()  # ParetoFront object
    # This is of type `Front` with `FrontValue` members.
    value_at_index_0 = front[0]  # FrontValue object
    all_values = front.values()  # list[FrontValue]

    # Get the members of the Pareto front:
    score = all_values[0].score() # list[float] - multi-objective score
    genotype = all_values[0].genotype()  # Genotype object
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

## Running 

Radiate provides multiple ways to run the `GeneticEngine`. 

1. **Run Method**

    The `run` method provides a more traditional way to run the engine. In rust it accepts a closure that takes the current epoch as an argument and returns a boolean indicating whether to stop the engine. In python, it accepts either a single limit or a list of limits that define the stopping conditions for the engine. The `run` method also accepts a `log`, `ui`, & `checkpoint` parameter to enable logging, a terminal UI, or checkpointing respectively.

2. **Iterator API** 

    The `GeneticEngine` is an inherently iterable concept, as such we can treat the engine as an iterator. Because of this we can use it in a `for` loop or with iterator methods like `map`, `filter`, etc. We can also extend the iterator with custom methods to provide additional functionality, such as running until a certain fitness (score) is reached, time limit, or convergence. These custom methods are essentially sytactic sugar for 'take_until' or 'skip_while' style iterators.

    During any sort of optimization task its useful to visually see the progress of the engine. Using the iterator API, we do this by calling `logging()` on the engine's iterator. This will give us nice console output of the progress provided by the [tracing](https://github.com/tokio-rs/tracing) project.

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

    # --- or using the engine's Run method with limits ---

    # Limits - run until a score target is reached
    score_limit = rd.ScoreLimit(0.01)
    generations_limit = rd.GenerationsLimit(100)
    seconds_limit = rd.SecondsLimit(60)
    # window and epsilon for convergence - how close the scores must be over the window to consider convergence
    convergence_limit = rd.ConvergenceLimit(window=50, epsilon=0.01) 

    # Log the progress of the engine to the console
    result = engine.run([
            score_limit,
            generations_limit,
            seconds_limit,
            convergence_limit
        ],
        log=True,
        ui=True, # Enable terminal UI - if enabled, log is ignored
        checkpoint=(10, "checkpoint.json") # checkpoint every 10 generations to 'checkpoint.json'
    )
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
    let result = engine.iter().until_score(target_score).last().unwrap();

    // 3.) run until a time limit is reached
    let time_limit = Duration::from_secs(60);
    let result = engine.iter().until_duration(time_limit).last().unwrap();

    // 4.) run until convergence
    let window = 50;
    let epsilon = 0.01; // how close the scores must be over the window to consider convergence
    let result = engine.iter().until_convergence(window, epsilon).last().unwrap();

    // 5.) log the progress of the engine to the console using the `logging()` method
    let result = engine.iter().logging().until_seconds(10).last().unwrap();

    // 5.) combined limits
    let result = engine
        .iter()
        .logging()
        .limit((
            Limit::Generation(100),
            Limit::Seconds(Duration::from_secs_f64(2.0)),
            Limit::Score(0.01),
        ))
        .last()
        .unwrap();

    // 6.) Checkpointing - save the engine state every 10 generations
    let checkpoint_path = "checkpoint.json";
    let result = engine
        .iter()
        .checkpoint(10, checkpoint_path) 
        .take(100)
        .last()
        .unwrap();

    // 7.) Using the engine's run method with a closure - stop after 100 generations
    let result = engine.run(|generation: &Generation<FloatChromosome>| {
        generation.index() >= 100
    });
    ```
---

## Control Interface

The engine provides a control interface that allows for pausing, resuming, and stopping the evolutionary process from external contexts. For instance, you might want to pause or step through generations from another thread or based on user input.

=== ":fontawesome-brands-python: Python"

    Not currently implemented.


=== ":fontawesome-brands-rust: Rust"

    ```rust
    use radiate::*;
    use std::thread;
    use std::time::Duration;

    let mut engine = GeneticEngine::builder()
        .minimizing()
        .codec(IntCodec::vector(5, 0..100))
        .fitness_fn(|geno: Vec<i32>| geno.iter().sum::<i32>())
        .build();

    let control = engine.control();

    let handle = thread::spawn(move || {
        let result = engine.iter().until_seconds(1_f64).last().unwrap();
        assert_eq!((result.seconds() - 1_f64).abs().round(), 0.0);
    });

    thread::sleep(Duration::from_millis(100));
    control.set_paused(true);

    // Ensure the engine is paused for at least 500ms
    thread::sleep(Duration::from_millis(500));
    control.set_paused(false);
    handle.join().unwrap();
    ```

---

## Tips

* Use appropriate population sizes (100-500 for most problems)
* Enable parallel execution for expensive fitness functions
* Consider species-based diversity for complex landscapes
* Experiment with different mutation and crossover rates
* Monitor convergence and adjust parameters dynamically
* Utilize logging and checkpointing for long runs
* Leverage the control interface for interactive runs


<!-- ```python
import radiate as rd
import threading
import time

# Create an engine
engine = rd.GeneticEngine(
    codec=rd.FloatCodec.scalar(0.0, 1.0), 
    fitness_fn=my_fitness_fn,  # Some fitness function
    # ... other parameters ...
)

# Get the control interface
control = engine.get_control()

# Run the engine in a separate thread
def run_engine():
    engine.run(rd.GenerationsLimit(1000))

engine_thread = threading.Thread(target=run_engine)
engine_thread.start()

# Pause the engine after 5 seconds
time.sleep(5)
control.pause()
print("Engine paused.")

# Resume the engine after another 5 seconds
time.sleep(5)
control.resume()
print("Engine resumed.")

# Stop the engine after another 5 seconds
time.sleep(5)
control.stop()
print("Engine stopped.")

engine_thread.join()
``` -->
