# Events and Subscriptions

Radiate provides an event system that allows you to monitor and react to the evolution process in real-time. This is great for:

- Tracking the progress of evolution
- Collecting metrics and statistics
- Implementing custom logging
- Visualizing the evolution process

## Overview

The event system in Radiate is built around the concept of event handlers or subscribers that can be attached to the `GeneticEngine`. These subscribers receive events at key points during the evolution process, allowing you to monitor and react to changes in the environment in real-time. The event system is designed to be flexible and extensible, allowing you to create custom event handlers that can perform various actions based on the evolution state.

The `GeneticEngine` tries its best to off-load almost the entire compute workload of the subscribers (handlers) to the user - be aware of this when implementing your handlers.

!!! note "Threading Behavior"
    
    Currently, the rust implementation is multi-threaded, meaning if you have multiple subscribers, there is no guarantee of the order in which they will be called. For python, the GIL locks the implementation into a single-thread, so subscribers will be called in the order they were added.

--- 
## Event Types

Radiate provides several key events that you can subscribe to:

??? note "Start Event"

    This event is triggered when the evolution process starts. It provides an opportunity to initialize any resources or perform setup tasks before the evolution begins.

??? note "Stop Event"

    This event is triggered when the evolution process stops, either due to reaching a stopping condition or being manually stopped. It provides access to:

    - The final `metrics` of the evolution
    - The best individual found
    - The final `score`, or fitness, of the best individual

??? note "Epoch Start Event"

    This event is triggered at the start of each generation (epoch) and provides the current generation number. It allows you to perform actions before the evolution step begins, such as resetting counters or logging initial state.

??? note "Epoch Complete Event"

    This event is triggered at the end of each generation (epoch) and provides information about:

    - The current generation number
    - The current `metrics` from the `GeneticEngine`
    - The best individual found from the `GeneticEngine` so far
    - The best `score`, or fitness, from the best individual

??? note "Step Start Event"

    This event is triggered at the start of each evolution step. It provides the name of the step being executed, allowing you to monitor the progress of specific steps in the evolution process.

??? note "Step Complete Event"

    This event is triggered at the end of each evolution step. It provides the name of the step that was executed, allowing you to log or monitor the completion of specific steps in the evolution process.

??? note "Engine Improvement Event"

    This event is triggered when the engine finds a new best individual during the evolution process. It provides:

    - The index of the generation where the improvement occurred
    - The best individual found at that point
    - The `score`, or fitness, of the best individual

---

## Subscribing to Events

You can subscribe to events in two ways:

### 1. Using a Callback Function

The simplest way to subscribe to events is by providing a callback function:


!!! warning ":construction: Under Construction :construction:"

    These docs are a work in progress and may not be complete or accurate. Please check back later for updates.

<!-- === ":fontawesome-brands-python: Python"

    ```python
    import radiate as rd

    def epoch_callback(generation):
        # Access generation information
        print(f"Generation {generation.number}:")
        print(f"Best score: {generation.score()}")
        print(f"Population size: {len(generation.population())}")
        
        # Access metrics
        metrics = generation.metrics()
        if metrics:
            print("Metrics:")
            for name, metric in metrics.iter():
                print(f"  {name}: {metric}")

    # Create and configure the engine
    engine = rd.GeneticEngine(
        codec=your_codec,
        fitness_func=your_fitness_func,
        # ... other parameters ...
    )

    # Subscribe to events
    engine.subscribe(epoch_callback)

    # Run the engine
    result = engine.run([rd.ScoreLimit(0.01)])
    ```

=== ":fontawesome-brands-rust: Rust"

    ```rust
    use radiate::*;

    fn epoch_callback(generation: &Epoch) {
        // Access generation information
        println!("Generation {}:", generation.index());
        println!("Best score: {:?}", generation.score());
        println!("Population size: {}", generation.population().len());
        
        // Access metrics
        if let Some(metrics) = generation.metrics() {
            println!("Metrics:");
            for (name, metric) in metrics.iter() {
                println!("  {}: {:?}", name, metric);
            }
        }
    }

    // Create and configure the engine
    let mut engine = GeneticEngine::builder()
        .codec(your_codec)
        .fitness_fn(your_fitness_fn)
        // ... other parameters ...
        .build();

    // Subscribe to events
    engine.subscribe(epoch_callback);

    // Run the engine
    let result = engine.run(|generation| {
        epoch_callback(generation);
        generation.score().as_f32() <= 0.01
    });
    ``` -->

### 2. Using an Event Handler Class

For more complex event handling, you can create a custom event handler class:


!!! warning ":construction: Under Construction :construction:"

    These docs are a work in progress and may not be complete or accurate. Please check back later for updates.

<!-- 
=== ":fontawesome-brands-python: Python"

    ```python
    import radiate as rd
    from radiate.handlers import EventHandler

    class CustomEventHandler(EventHandler):
        def __init__(self):
            self.best_score = float('inf')
            self.generations_without_improvement = 0
        
        def on_event(self, generation):
            # Track best score
            current_score = generation.score()
            if current_score < self.best_score:
                self.best_score = current_score
                self.generations_without_improvement = 0
            else:
                self.generations_without_improvement += 1
            
            # Print progress
            print(f"Generation {generation.number}:")
            print(f"Current score: {current_score}")
            print(f"Best score: {self.best_score}")
            print(f"Generations without improvement: {self.generations_without_improvement}")
            
            # Access species information if diversity is enabled
            if generation.species():
                print(f"Number of species: {len(generation.species())}")

    # Create and configure the engine
    engine = rd.GeneticEngine(
        codec=your_codec,
        fitness_func=your_fitness_func,
        # ... other parameters ...
    )

    # Subscribe using the event handler
    handler = CustomEventHandler()
    engine.subscribe(handler)

    # Run the engine
    result = engine.run([rd.ScoreLimit(0.01)])
    ```

=== ":fontawesome-brands-rust: Rust"

    ```rust
    use radiate::*;
    use std::sync::Arc;

    struct CustomEventHandler {
        best_score: f32,
        generations_without_improvement: usize,
    }

    impl CustomEventHandler {
        fn new() -> Self {
            Self {
                best_score: f32::INFINITY,
                generations_without_improvement: 0,
            }
        }
        
        fn handle_event(&mut self, generation: &Epoch) {
            // Track best score
            let current_score = generation.score().as_f32();
            if current_score < self.best_score {
                self.best_score = current_score;
                self.generations_without_improvement = 0;
            } else {
                self.generations_without_improvement += 1;
            }
            
            // Print progress
            println!("Generation {}:", generation.index());
            println!("Current score: {}", current_score);
            println!("Best score: {}", self.best_score);
            println!("Generations without improvement: {}", self.generations_without_improvement);
            
            // Access species information if diversity is enabled
            if let Some(species) = generation.species() {
                println!("Number of species: {}", species.len());
            }
        }
    }

    // Create and configure the engine
    let mut engine = GeneticEngine::builder()
        .codec(your_codec)
        .fitness_fn(your_fitness_fn)
        // ... other parameters ...
        .build();

    // Create and use the event handler
    let mut handler = CustomEventHandler::new();
    engine.subscribe(Arc::new(move |generation| {
        handler.handle_event(generation);
    }));

    // Run the engine
    let result = engine.run(|generation| {
        generation.score().as_f32() <= 0.01
    });
    ``` -->

## Best Practices

1. **Keep Event Handlers Light**:
    - Event handlers are called frequently during evolution
    - Avoid heavy computations in event handlers

2. **Use Multiple Subscribers**:
    - You can subscribe multiple handlers to the same engine
    - Separate concerns into different handlers
        - Example: one for logging, one for metrics, one for visualization

3. **Handle Errors Gracefully**:
    - Event handlers should not crash the evolution process
    - Log errors instead of raising exceptions - do not expect the `GeneticEngine` to throw exceptions

4. **Monitor Performance**:
    - Be aware that event handling adds some overhead depending on your implementation
    - Use built in `metrics` to track certain metrics or performance characteristics if possible
    - Be cautious of your implementation - consider disabling event handling in production if not essential


<!-- ## Available Metrics

The event system provides access to various metrics through the `metrics()` method. Here are some of the key metrics available:

- `age`: The age of individuals in the population
- `score`: The fitness scores of individuals
- `genome_size`: The size of genomes in the population
- `unique_scores`: The number of unique fitness scores
- `unique_members`: The number of unique individuals
- `species_age`: The age of species (if diversity is enabled)
- `evolution_time`: The time taken for evolution -->

<!-- ## Example: Complete Monitoring Setup

Here's a complete example showing how to set up comprehensive monitoring:

=== ":fontawesome-brands-python: Python"

    ```python
    import radiate as rd
    from radiate.handlers import EventHandler
    import json
    from datetime import datetime

    class MonitoringHandler(EventHandler):
        def __init__(self, log_file="evolution.log"):
            self.log_file = log_file
            self.start_time = datetime.now()
            self.best_score = float('inf')
            self.generations_without_improvement = 0
            
            # Initialize log file
            with open(self.log_file, 'w') as f:
                f.write("Evolution Log\n")
                f.write("=============\n\n")
        
        def on_event(self, generation):
            # Calculate metrics
            current_score = generation.score()
            time_elapsed = (datetime.now() - self.start_time).total_seconds()
            
            # Update best score tracking
            if current_score < self.best_score:
                self.best_score = current_score
                self.generations_without_improvement = 0
            else:
                self.generations_without_improvement += 1
            
            # Collect metrics
            metrics = {
                "generation": generation.number,
                "current_score": current_score,
                "best_score": self.best_score,
                "time_elapsed": time_elapsed,
                "population_size": len(generation.population()),
                "generations_without_improvement": self.generations_without_improvement
            }
            
            # Add species information if available
            if generation.species():
                metrics["species_count"] = len(generation.species())
                metrics["species_ages"] = [
                    species.age(generation.number)
                    for species in generation.species()
                ]
            
            # Add other metrics from the generation
            for name, metric in generation.metrics().iter():
                metrics[name] = metric.distribution_mean()
            
            # Log to file
            with open(self.log_file, 'a') as f:
                f.write(f"\nGeneration {generation.number}:\n")
                f.write(json.dumps(metrics, indent=2))
                f.write("\n")
            
            # Print progress
            print(f"Generation {generation.number}:")
            print(f"  Score: {current_score:.6f}")
            print(f"  Best: {self.best_score:.6f}")
            print(f"  Time: {time_elapsed:.1f}s")
            print(f"  Species: {metrics.get('species_count', 'N/A')}")

    # Create and configure the engine
    engine = rd.GeneticEngine(
        codec=your_codec,
        fitness_func=your_fitness_func,
        diversity=rd.EuclideanDistance(),
        species_threshold=0.5,
        # ... other parameters ...
    )

    # Set up monitoring
    monitor = MonitoringHandler("evolution.log")
    engine.subscribe(monitor)

    # Run the engine
    result = engine.run([
        rd.ScoreLimit(0.01),
        rd.GenerationsLimit(1000)
    ])
    ```

=== ":fontawesome-brands-rust: Rust"

    ```rust
    use radiate::*;
    use std::{
        fs::File,
        io::Write,
        sync::Arc,
        time::Instant,
    };
    use serde_json::json;

    struct MonitoringHandler {
        log_file: String,
        start_time: Instant,
        best_score: f32,
        generations_without_improvement: usize,
    }

    impl MonitoringHandler {
        fn new(log_file: &str) -> Self {
            // Initialize log file
            let mut file = File::create(log_file).unwrap();
            writeln!(file, "Evolution Log").unwrap();
            writeln!(file, "=============\n").unwrap();
            
            Self {
                log_file: log_file.to_string(),
                start_time: Instant::now(),
                best_score: f32::INFINITY,
                generations_without_improvement: 0,
            }
        }
        
        fn handle_event(&mut self, generation: &Epoch) {
            // Calculate metrics
            let current_score = generation.score().as_f32();
            let time_elapsed = self.start_time.elapsed().as_secs_f64();
            
            // Update best score tracking
            if current_score < self.best_score {
                self.best_score = current_score;
                self.generations_without_improvement = 0;
            } else {
                self.generations_without_improvement += 1;
            }
            
            // Collect metrics
            let mut metrics = json!({
                "generation": generation.index(),
                "current_score": current_score,
                "best_score": self.best_score,
                "time_elapsed": time_elapsed,
                "population_size": generation.population().len(),
                "generations_without_improvement": self.generations_without_improvement
            });
            
            // Add species information if available
            if let Some(species) = generation.species() {
                metrics["species_count"] = json!(species.len());
                metrics["species_ages"] = json!(
                    species.iter()
                        .map(|s| s.age(generation.index()))
                        .collect::<Vec<_>>()
                );
            }
            
            // Add other metrics from the generation
            if let Some(metrics_set) = generation.metrics() {
                for (name, metric) in metrics_set.iter() {
                    if let Some(mean) = metric.distribution_mean() {
                        metrics[name] = json!(mean);
                    }
                }
            }
            
            // Log to file
            let mut file = File::options()
                .append(true)
                .open(&self.log_file)
                .unwrap();
            
            writeln!(file, "\nGeneration {}:", generation.index()).unwrap();
            writeln!(file, "{}", serde_json::to_string_pretty(&metrics).unwrap()).unwrap();
            
            // Print progress
            println!("Generation {}:", generation.index());
            println!("  Score: {:.6}", current_score);
            println!("  Best: {:.6}", self.best_score);
            println!("  Time: {:.1}s", time_elapsed);
            println!("  Species: {}", 
                metrics.get("species_count")
                    .map_or("N/A", |v| v.as_str().unwrap_or("N/A"))
            );
        }
    }

    // Create and configure the engine
    let mut engine = GeneticEngine::builder()
        .codec(your_codec)
        .fitness_fn(your_fitness_fn)
        .diversity(EuclideanDistance::new())
        .species_threshold(0.5)
        // ... other parameters ...
        .build();

    // Set up monitoring
    let mut monitor = MonitoringHandler::new("evolution.log");
    engine.subscribe(Arc::new(move |generation| {
        monitor.handle_event(generation);
    }));

    // Run the engine
    let result = engine.run(|generation| {
        generation.index() >= 1000 || generation.score().as_f32() <= 0.01
    });
    ``` -->

<!-- This example demonstrates:
- Comprehensive metric collection
- File-based logging
- Progress monitoring
- Species tracking
- Performance measurement
- Best score tracking
- Early stopping conditions

The logged data can be used for:
- Post-evolution analysis
- Visualization
- Performance optimization
- Debugging
- Documentation of evolution runs -->